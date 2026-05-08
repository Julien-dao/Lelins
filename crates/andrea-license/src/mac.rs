//! Truncated HMAC-SHA256 over the license payload.
//!
//! `compute_mac(secret, packed_payload) → u32` returns the first
//! [`crate::MAC_BITS`] bits packed into the low-order bits of a u32.
//! Truncation is to 28 bits — explicitly **not** a cryptographic-strength
//! signature. See `crate` docs for the threat model.
//!
//! HMAC-SHA256 was chosen over keyed BLAKE2b for API simplicity in the
//! `hmac` crate. The security properties are equivalent for our use case
//! (deterrence against casual sharing, with the secret embedded in the
//! binary).

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::MAC_BITS;

type HmacSha256 = Hmac<Sha256>;

/// Compute the truncated HMAC-SHA256 MAC over a packed payload.
///
/// `packed72` must be the output of [`crate::LicensePayload::pack72`] — i.e.
/// the payload sits in bits `[MAC_BITS..TOTAL_BITS)` and the MAC zone
/// (low `MAC_BITS` bits) is zero.
///
/// The full 16-byte big-endian representation of `packed72` is hashed: the
/// MAC zone being zero ensures the input depends only on the payload bits.
pub fn compute_mac(server_secret: &[u8], packed72: u128) -> u32 {
    debug_assert!(
        packed72 & ((1u128 << MAC_BITS) - 1) == 0,
        "MAC zone (low {MAC_BITS} bits) must be zero before MAC computation"
    );
    debug_assert!(
        packed72 >> crate::TOTAL_BITS == 0,
        "packed payload must fit within TOTAL_BITS bits"
    );

    let mut mac = HmacSha256::new_from_slice(server_secret).expect("HMAC accepts any key length");
    let payload_bytes = packed72.to_be_bytes(); // 16 bytes
    mac.update(&payload_bytes);

    let tag = mac.finalize().into_bytes();
    let raw = u32::from_be_bytes([tag[0], tag[1], tag[2], tag[3]]);

    // Keep the high MAC_BITS of the 32-bit raw, mask to MAC_BITS.
    let mask = if MAC_BITS == 32 {
        u32::MAX
    } else {
        (1u32 << MAC_BITS) - 1
    };
    (raw >> (32 - MAC_BITS)) & mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use subtle::ConstantTimeEq;

    /// Build a valid `packed72` value (payload bits in [MAC_BITS..TOTAL_BITS),
    /// MAC zone clear) for testing.
    fn make_packed(payload_low72: u128) -> u128 {
        debug_assert!(payload_low72 >> 72 == 0);
        payload_low72 << MAC_BITS
    }

    #[test]
    fn mac_fits_in_28_bits() {
        let mac = compute_mac(b"andrea-secret", make_packed(0x1234_5678_9ABC_DEF0));
        assert_eq!(mac >> MAC_BITS, 0);
    }

    #[test]
    fn mac_is_deterministic() {
        let secret = b"server-secret-for-andrea";
        let packed = make_packed(0xCAFEBABE);
        let m1 = compute_mac(secret, packed);
        let m2 = compute_mac(secret, packed);
        assert_eq!(m1, m2);
    }

    #[test]
    fn mac_changes_when_secret_changes() {
        let packed = make_packed(0xCAFEBABE);
        let a = compute_mac(b"secret-a", packed);
        let b = compute_mac(b"secret-b", packed);
        assert_ne!(a, b);
    }

    #[test]
    fn mac_changes_when_payload_changes() {
        let secret = b"shared-secret";
        let a = compute_mac(secret, make_packed(0x1111));
        let b = compute_mac(secret, make_packed(0x2222));
        assert_ne!(a, b);
    }

    #[test]
    fn constant_time_eq_works_on_macs() {
        let a: u32 = 0x0FFF_FFFF;
        let b: u32 = 0x0FFF_FFFF;
        assert!(bool::from(a.to_be_bytes().ct_eq(&b.to_be_bytes())));
        let c: u32 = 0x0FFF_FFFE;
        assert!(!bool::from(a.to_be_bytes().ct_eq(&c.to_be_bytes())));
    }
}
