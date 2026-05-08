//! ANDREA license keys.
//!
//! Format: `ANDREA-{TIER}-XXXXX-XXXXX-XXXXX-XXXXX`
//!
//! Payload layout (100 bits, Crockford Base32 encoded over 20 characters):
//!
//! | Field        | Bits | Range / Meaning                                  |
//! |--------------|------|--------------------------------------------------|
//! | version      | 4    | format version (currently 1)                     |
//! | tier         | 4    | 0=Discovery, 1=Pro, 2=Master, 3=Bundle           |
//! | email_hash   | 32   | first 4 bytes of `SHA256(lowercase(email))`      |
//! | issued_days  | 16   | days since 2026-01-01 (~179 years range)         |
//! | features     | 16   | bitmask of fine-grained feature flags            |
//! | mac          | 28   | `BLAKE2b(server_secret, payload)[..28 bits]`     |
//!
//! Threat model: deterrence against casual sharing, **not** cryptographic
//! invulnerability. The server secret is embedded in the binary; once
//! extracted, an attacker can forge keys. The `email_hash` ties a key to a
//! specific buyer, limiting damage from individual leaks. Compromise mitigation
//! is via revocation list shipped with each release.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod crockford;
mod mac;
mod payload;
mod verifier;

pub use crockford::{decode_payload, encode_payload, normalize, CrockfordError};
pub use mac::compute_mac;
pub use payload::{LicensePayload, Tier, TierError};
pub use verifier::{generate, verify, License, LicenseError};

/// Number of payload bits (excluding MAC).
pub const PAYLOAD_BITS: usize = 72;
/// Number of MAC bits (truncated BLAKE2b).
pub const MAC_BITS: usize = 28;
/// Total bits encoded in the human-readable key (payload + MAC).
pub const TOTAL_BITS: usize = PAYLOAD_BITS + MAC_BITS;
/// Total characters in the Crockford-encoded portion.
pub const ENCODED_CHARS: usize = TOTAL_BITS / 5;

/// Current format version (in the version field of the payload).
pub const CURRENT_VERSION: u8 = 1;

/// Compute the email hash used in the payload.
///
/// Lowercases the email, trims surrounding whitespace, then takes the first
/// 4 bytes of `SHA256`.
pub fn email_hash(email: &str) -> u32 {
    use sha2::{Digest, Sha256};
    let normalized = email.trim().to_lowercase();
    let digest = Sha256::digest(normalized.as_bytes());
    u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_hash_is_case_insensitive_and_trims() {
        let h1 = email_hash("Julien@Andrea-Formation.fr");
        let h2 = email_hash("julien@andrea-formation.fr");
        let h3 = email_hash("  julien@andrea-formation.fr  ");
        assert_eq!(h1, h2);
        assert_eq!(h2, h3);
    }

    #[test]
    fn different_emails_have_different_hashes() {
        let a = email_hash("a@example.com");
        let b = email_hash("b@example.com");
        assert_ne!(a, b);
    }

    #[test]
    fn constants_are_consistent() {
        assert_eq!(PAYLOAD_BITS + MAC_BITS, TOTAL_BITS);
        assert_eq!(TOTAL_BITS % 5, 0);
        assert_eq!(ENCODED_CHARS, 20);
    }
}
