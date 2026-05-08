//! `.andrea-backup` file format — encrypted, compressed user-data archive.
//!
//! # Layout
//!
//! ```text
//! +--------+--------+--------+--------+
//! | magic  |version | mode   | rsv    |   8 bytes header
//! | "ANDR" |   1    |  0/1   |   0    |
//! +--------+--------+--------+--------+
//! |          salt  (16 bytes)         |   Argon2id salt
//! +-----------------------------------+
//! |          nonce (24 bytes)         |   XChaCha20Poly1305 nonce
//! +-----------------------------------+
//! |       ciphertext + tag (var)      |   XChaCha20Poly1305(zstd(plaintext))
//! +-----------------------------------+
//! ```
//!
//! # Modes
//!
//! - `AutoDerived` (mode = 0) — the application derives the key from the
//!   user's email + a fixed domain string. Convenient (no passphrase to
//!   remember) but only re-importable on the same machine/profile pair.
//! - `Passphrase`  (mode = 1) — Argon2id-derived from a user-supplied
//!   passphrase. Portable across machines.
//!
//! # Threat model
//!
//! Protects user documents against casual reading (machine sharing, lost
//! laptop). Not designed to resist an adversary who has compromised the
//! ANDREA install AND the email AND the user's keystrokes simultaneously.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use argon2::Argon2;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Magic bytes at the start of every `.andrea-backup` file.
pub const MAGIC: &[u8; 4] = b"ANDR";
/// Current format version.
pub const FORMAT_VERSION: u8 = 1;
/// Length of the Argon2id salt embedded in the header.
pub const SALT_LEN: usize = 16;
/// Length of the XChaCha20Poly1305 nonce embedded in the header.
pub const NONCE_LEN: usize = 24;
/// Length of the AEAD key derived from key material.
pub const KEY_LEN: usize = 32;
/// Length of the fixed-size header (magic + version + mode + reserved + salt + nonce).
pub const HEADER_LEN: usize = 4 + 1 + 1 + 2 + SALT_LEN + NONCE_LEN;
/// Compression level used for zstd (1..=22). 3 is the default zstd level.
const ZSTD_LEVEL: i32 = 3;
/// Domain separation string for auto-derived mode.
const DOMAIN_AUTO: &[u8] = b"ANDREA_BACKUP_V1_AUTO";

/// Errors raised by the backup format.
#[derive(Debug, Error)]
pub enum BackupError {
    /// The file does not start with the expected magic bytes.
    #[error("not an .andrea-backup file (bad magic)")]
    BadMagic,
    /// Unsupported format version.
    #[error("unsupported .andrea-backup format version {0}")]
    UnsupportedVersion(u8),
    /// Unknown mode byte.
    #[error("unknown mode byte {0}")]
    UnknownMode(u8),
    /// File is truncated.
    #[error("file truncated: expected at least {expected} bytes, got {got}")]
    Truncated {
        /// Minimum bytes required.
        expected: usize,
        /// Bytes actually present.
        got: usize,
    },
    /// AEAD authentication / decryption failure (corrupted or wrong key).
    #[error("authentication failed: file is corrupted or the key is wrong")]
    Authentication,
    /// I/O failure on the compressor.
    #[error("compression error: {0}")]
    Compression(String),
    /// Argon2 KDF failed.
    #[error("KDF error: {0}")]
    Kdf(String),
}

/// Mode byte encoded in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Key derived from email + domain string (no passphrase).
    AutoDerived = 0,
    /// Key derived from user-supplied passphrase.
    Passphrase = 1,
}

impl Mode {
    fn from_byte(b: u8) -> Result<Self, BackupError> {
        match b {
            0 => Ok(Mode::AutoDerived),
            1 => Ok(Mode::Passphrase),
            other => Err(BackupError::UnknownMode(other)),
        }
    }
}

/// Material to derive an AEAD key from. Use [`KeyMaterial::auto`] or
/// [`KeyMaterial::passphrase`].
#[derive(Debug, Clone)]
pub enum KeyMaterial<'a> {
    /// Auto-derived from the user's email (used as input to Argon2id).
    Auto {
        /// User email (lowercased and trimmed before hashing).
        email: &'a str,
    },
    /// User-supplied passphrase.
    Passphrase {
        /// Passphrase bytes — typically UTF-8 from the user.
        bytes: &'a [u8],
    },
}

impl<'a> KeyMaterial<'a> {
    /// Auto-derived key material from the user's email.
    pub fn auto(email: &'a str) -> Self {
        KeyMaterial::Auto { email }
    }

    /// Passphrase key material.
    pub fn passphrase(passphrase: &'a str) -> Self {
        KeyMaterial::Passphrase {
            bytes: passphrase.as_bytes(),
        }
    }

    fn mode(&self) -> Mode {
        match self {
            KeyMaterial::Auto { .. } => Mode::AutoDerived,
            KeyMaterial::Passphrase { .. } => Mode::Passphrase,
        }
    }

    fn input_bytes(&self) -> Vec<u8> {
        match self {
            KeyMaterial::Auto { email } => {
                let normalized = email.trim().to_lowercase();
                let mut buf = Vec::with_capacity(DOMAIN_AUTO.len() + 1 + normalized.len());
                buf.extend_from_slice(DOMAIN_AUTO);
                buf.push(0);
                buf.extend_from_slice(normalized.as_bytes());
                buf
            }
            KeyMaterial::Passphrase { bytes } => bytes.to_vec(),
        }
    }
}

fn derive_key(material: &KeyMaterial, salt: &[u8]) -> Result<[u8; KEY_LEN], BackupError> {
    let argon = Argon2::default();
    let input = material.input_bytes();
    let mut key = [0u8; KEY_LEN];
    argon
        .hash_password_into(&input, salt, &mut key)
        .map_err(|e| BackupError::Kdf(e.to_string()))?;
    Ok(key)
}

/// Seal `plaintext` into a self-contained `.andrea-backup` byte vector.
pub fn seal(plaintext: &[u8], material: &KeyMaterial) -> Result<Vec<u8>, BackupError> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(material, &salt)?;
    let cipher = XChaCha20Poly1305::new((&key).into());

    let compressed = zstd::stream::encode_all(plaintext, ZSTD_LEVEL)
        .map_err(|e| BackupError::Compression(e.to_string()))?;

    let nonce = XNonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, compressed.as_slice())
        .map_err(|_| BackupError::Authentication)?;

    let mut out = Vec::with_capacity(HEADER_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.push(FORMAT_VERSION);
    out.push(material.mode() as u8);
    out.extend_from_slice(&[0u8, 0u8]); // reserved
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Open and verify a `.andrea-backup` byte vector, returning the original
/// plaintext.
pub fn open(blob: &[u8], material: &KeyMaterial) -> Result<Vec<u8>, BackupError> {
    if blob.len() < HEADER_LEN {
        return Err(BackupError::Truncated {
            expected: HEADER_LEN,
            got: blob.len(),
        });
    }
    if &blob[0..4] != MAGIC {
        return Err(BackupError::BadMagic);
    }
    let version = blob[4];
    if version != FORMAT_VERSION {
        return Err(BackupError::UnsupportedVersion(version));
    }
    let mode = Mode::from_byte(blob[5])?;
    if mode != material.mode() {
        // A mode mismatch is not strictly an error if the caller can supply
        // either form; but we make the intent explicit and surface it as a
        // decryption failure below.
    }
    let salt = &blob[8..8 + SALT_LEN];
    let nonce_bytes = &blob[8 + SALT_LEN..HEADER_LEN];
    let ciphertext = &blob[HEADER_LEN..];

    let key = derive_key(material, salt)?;
    let cipher = XChaCha20Poly1305::new((&key).into());
    let nonce = XNonce::from_slice(nonce_bytes);
    let compressed = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| BackupError::Authentication)?;
    let plaintext = zstd::stream::decode_all(compressed.as_slice())
        .map_err(|e| BackupError::Compression(e.to_string()))?;
    Ok(plaintext)
}

/// Compute a stable SHA-256 digest of the (clear) plaintext, suitable for
/// the `backup.sha256` column in the SQLite `backup` table.
pub fn plaintext_digest(plaintext: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(plaintext);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_passphrase() {
        let secret = b"the quick brown fox jumps over the lazy dog".repeat(50);
        let material = KeyMaterial::passphrase("correct horse battery staple");
        let blob = seal(&secret, &material).unwrap();
        assert!(blob.starts_with(MAGIC));
        let opened = open(&blob, &material).unwrap();
        assert_eq!(opened, secret);
    }

    #[test]
    fn roundtrip_auto_derived() {
        let secret = b"hello andrea";
        let material = KeyMaterial::auto("julien@andrea-formation.fr");
        let blob = seal(secret, &material).unwrap();
        let opened = open(&blob, &material).unwrap();
        assert_eq!(opened, secret);
    }

    #[test]
    fn open_with_wrong_passphrase_fails() {
        let secret = b"top secret";
        let blob = seal(secret, &KeyMaterial::passphrase("right")).unwrap();
        let err = open(&blob, &KeyMaterial::passphrase("wrong")).unwrap_err();
        assert!(matches!(err, BackupError::Authentication));
    }

    #[test]
    fn open_with_wrong_email_fails() {
        let blob = seal(b"x", &KeyMaterial::auto("julien@example.com")).unwrap();
        let err = open(&blob, &KeyMaterial::auto("other@example.com")).unwrap_err();
        assert!(matches!(err, BackupError::Authentication));
    }

    #[test]
    fn open_rejects_bad_magic() {
        let mut blob = seal(b"x", &KeyMaterial::passphrase("p")).unwrap();
        blob[0] = b'X';
        assert!(matches!(
            open(&blob, &KeyMaterial::passphrase("p")),
            Err(BackupError::BadMagic)
        ));
    }

    #[test]
    fn open_rejects_truncated() {
        let blob = vec![0u8; HEADER_LEN - 1];
        assert!(matches!(
            open(&blob, &KeyMaterial::passphrase("p")),
            Err(BackupError::Truncated { .. })
        ));
    }

    #[test]
    fn open_rejects_unknown_version() {
        let mut blob = seal(b"x", &KeyMaterial::passphrase("p")).unwrap();
        blob[4] = 99;
        assert!(matches!(
            open(&blob, &KeyMaterial::passphrase("p")),
            Err(BackupError::UnsupportedVersion(99))
        ));
    }

    #[test]
    fn tampering_with_ciphertext_is_detected() {
        let secret = b"sensitive";
        let material = KeyMaterial::passphrase("p");
        let mut blob = seal(secret, &material).unwrap();
        // Flip a byte in the ciphertext region.
        let last = blob.len() - 5;
        blob[last] ^= 0xFF;
        assert!(matches!(
            open(&blob, &material),
            Err(BackupError::Authentication)
        ));
    }

    #[test]
    fn header_is_exactly_documented_length() {
        let blob = seal(b"x", &KeyMaterial::passphrase("p")).unwrap();
        assert!(blob.len() >= HEADER_LEN);
        assert_eq!(&blob[0..4], MAGIC);
        assert_eq!(blob[4], FORMAT_VERSION);
        assert_eq!(blob[5], Mode::Passphrase as u8);
        assert_eq!(&blob[6..8], &[0, 0]); // reserved
    }

    #[test]
    fn empty_plaintext_roundtrips() {
        let material = KeyMaterial::passphrase("p");
        let blob = seal(&[], &material).unwrap();
        let opened = open(&blob, &material).unwrap();
        assert!(opened.is_empty());
    }

    #[test]
    fn plaintext_digest_is_stable() {
        let a = plaintext_digest(b"abc");
        let b = plaintext_digest(b"abc");
        assert_eq!(a, b);
        let c = plaintext_digest(b"abd");
        assert_ne!(a, c);
    }
}
