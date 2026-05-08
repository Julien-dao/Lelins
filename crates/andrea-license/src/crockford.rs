//! Crockford Base32 encoder/decoder for ANDREA license payloads.
//!
//! Reference: <https://www.crockford.com/base32.html>
//!
//! Alphabet: `0123456789ABCDEFGHJKMNPQRSTVWXYZ` (32 symbols, excluding `I`,
//! `L`, `O`, `U` to avoid visual ambiguity with `1`, `1`, `0`, and obscenities).
//!
//! Decoding is case-insensitive and tolerant: `I`/`i` → `1`, `L`/`l` → `1`,
//! `O`/`o` → `0`. Any non-alphabet character (including hyphens and spaces)
//! is silently skipped.

use super::{ENCODED_CHARS, TOTAL_BITS};
use thiserror::Error;

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Errors raised while decoding a Crockford payload.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CrockfordError {
    /// The decoded payload did not contain the expected number of characters.
    #[error("expected {expected} encoded chars, got {got}")]
    WrongLength {
        /// Expected characters (`ENCODED_CHARS`).
        expected: usize,
        /// Characters actually found after normalization.
        got: usize,
    },
    /// A character outside the Crockford alphabet was encountered.
    #[error("invalid character `{0}` in license key")]
    InvalidCharacter(char),
    /// The integer payload exceeds `TOTAL_BITS` bits.
    #[error("payload value exceeds {TOTAL_BITS} bits")]
    PayloadOverflow,
}

/// Encode a `u128` payload (using the lowest `TOTAL_BITS` bits) into a string
/// of [`ENCODED_CHARS`] uppercase Crockford characters, no separators.
pub fn encode_payload(payload: u128) -> Result<String, CrockfordError> {
    if TOTAL_BITS < 128 && (payload >> TOTAL_BITS) != 0 {
        return Err(CrockfordError::PayloadOverflow);
    }
    let mut out = Vec::with_capacity(ENCODED_CHARS);
    for i in 0..ENCODED_CHARS {
        let shift = TOTAL_BITS - 5 - i * 5;
        let idx = ((payload >> shift) & 0b11111) as usize;
        out.push(ALPHABET[idx]);
    }
    Ok(String::from_utf8(out).expect("alphabet is ASCII"))
}

/// Normalize an arbitrary user-typed string into uppercase Crockford
/// characters, stripping separators and applying the Crockford tolerance
/// rules (`I`/`L` → `1`, `O` → `0`).
pub fn normalize(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        let mapped = match ch {
            ' ' | '-' | '_' | '\t' | '\n' | '\r' => continue,
            'I' | 'i' | 'L' | 'l' => '1',
            'O' | 'o' => '0',
            c => c.to_ascii_uppercase(),
        };
        out.push(mapped);
    }
    out
}

/// Decode a Crockford-encoded string back to a `u128` payload (the lowest
/// `TOTAL_BITS` bits are populated).
///
/// The input may contain hyphens, spaces, and any of the tolerance-mapped
/// characters; they are normalized first.
pub fn decode_payload(raw: &str) -> Result<u128, CrockfordError> {
    let normalized = normalize(raw);
    if normalized.len() != ENCODED_CHARS {
        return Err(CrockfordError::WrongLength {
            expected: ENCODED_CHARS,
            got: normalized.len(),
        });
    }
    let mut payload: u128 = 0;
    for ch in normalized.chars() {
        let value = char_to_value(ch)?;
        payload = (payload << 5) | (value as u128);
    }
    Ok(payload)
}

fn char_to_value(c: char) -> Result<u8, CrockfordError> {
    let byte = c as u8;
    let pos = ALPHABET
        .iter()
        .position(|&b| b == byte)
        .ok_or(CrockfordError::InvalidCharacter(c))?;
    Ok(pos as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_zero() {
        let s = encode_payload(0).unwrap();
        assert_eq!(s.len(), ENCODED_CHARS);
        assert!(s.chars().all(|c| c == '0'));
        assert_eq!(decode_payload(&s).unwrap(), 0);
    }

    #[test]
    fn roundtrip_max() {
        let max = (1u128 << TOTAL_BITS) - 1;
        let s = encode_payload(max).unwrap();
        assert_eq!(decode_payload(&s).unwrap(), max);
    }

    #[test]
    fn roundtrip_random_values() {
        let cases = [0x123456789ABCDEFu128, 0xDEADBEEFu128, 0x0F0F0F0F0F0F0Fu128];
        for &p in &cases {
            let s = encode_payload(p).unwrap();
            assert_eq!(decode_payload(&s).unwrap(), p, "case {p:#x}");
        }
    }

    #[test]
    fn rejects_overflow() {
        let too_big = 1u128 << TOTAL_BITS;
        assert_eq!(
            encode_payload(too_big),
            Err(CrockfordError::PayloadOverflow)
        );
    }

    #[test]
    fn normalize_handles_separators_and_tolerance() {
        // Crockford tolerance maps O→0 and I/L→1 even inside the brand prefix —
        // tier parsing accepts both the canonical and the normalized forms.
        assert_eq!(normalize("ANDREA-DECO-12345-67890"), "ANDREADEC01234567890");
        assert_eq!(normalize("abc-def"), "ABCDEF");
        assert_eq!(normalize("io1l0"), "10110");
        assert_eq!(normalize("  hello  "), "HE110");
    }

    #[test]
    fn decode_with_hyphens_and_lowercase() {
        let original = encode_payload(0xCAFEBABE).unwrap();
        // Insert hyphens every 5 chars and lowercase
        let mut formatted = String::new();
        for (i, ch) in original.chars().enumerate() {
            if i > 0 && i % 5 == 0 {
                formatted.push('-');
            }
            formatted.push(ch.to_ascii_lowercase());
        }
        assert_eq!(decode_payload(&formatted).unwrap(), 0xCAFEBABE);
    }

    #[test]
    fn decode_rejects_invalid_char() {
        // `U` is excluded from the Crockford alphabet (no tolerance mapping).
        let bad = "00000000000000000000"
            .chars()
            .enumerate()
            .map(|(i, c)| if i == 5 { 'U' } else { c })
            .collect::<String>();
        assert_eq!(
            decode_payload(&bad),
            Err(CrockfordError::InvalidCharacter('U'))
        );
    }

    #[test]
    fn decode_rejects_wrong_length() {
        let short = "0".repeat(ENCODED_CHARS - 1);
        assert!(matches!(
            decode_payload(&short),
            Err(CrockfordError::WrongLength { .. })
        ));
        let long = "0".repeat(ENCODED_CHARS + 1);
        assert!(matches!(
            decode_payload(&long),
            Err(CrockfordError::WrongLength { .. })
        ));
    }
}
