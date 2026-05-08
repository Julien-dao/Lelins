//! License payload : the 72 bits of structured data signed by the MAC.
//!
//! Layout (most significant bit first):
//!
//! ```text
//!  4   4    32          16          16
//! [ver][tier][email_hash][issued_days][features]
//! ```

use thiserror::Error;

/// License tier, encoded in 4 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tier {
    /// ANDREA Découverte — base.
    Discovery = 0,
    /// ANDREA Pro — adds evaluator + DP guide.
    Pro = 1,
    /// ANDREA Maître — adds cohort tracking + Qualiopi exports.
    Master = 2,
    /// "Le Titre en main" bundle — equivalent to Master.
    Bundle = 3,
}

/// Errors raised while parsing a tier.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TierError {
    /// Numeric value does not match a known tier.
    #[error("unknown tier code {0}")]
    UnknownCode(u8),
    /// Textual prefix in a human-readable key does not match a known tier.
    #[error("unknown tier label `{0}`")]
    UnknownLabel(String),
}

impl Tier {
    /// Parse the textual prefix used in the human-readable key
    /// (`DECO`, `PRO`, `MAIT`, `BNDL`).
    ///
    /// Accepts both the canonical form and the Crockford-normalized form
    /// (where `O`→`0`, `L`→`1`, `I`→`1`) so that a user-typed key surviving
    /// a [`crate::normalize`] pass still parses correctly.
    pub fn from_label(label: &str) -> Result<Self, TierError> {
        let upper = label.to_ascii_uppercase();
        match upper.as_str() {
            "DECO" | "DEC0" => Ok(Tier::Discovery),
            "PRO" | "PR0" => Ok(Tier::Pro),
            "MAIT" | "MA1T" => Ok(Tier::Master),
            "BNDL" | "BND1" => Ok(Tier::Bundle),
            _ => Err(TierError::UnknownLabel(upper)),
        }
    }

    /// Render the textual prefix for the human-readable key.
    pub fn label(self) -> &'static str {
        match self {
            Tier::Discovery => "DECO",
            Tier::Pro => "PRO",
            Tier::Master => "MAIT",
            Tier::Bundle => "BNDL",
        }
    }

    /// Convert from the 4-bit numeric code stored in the payload.
    pub fn from_code(code: u8) -> Result<Self, TierError> {
        match code {
            0 => Ok(Tier::Discovery),
            1 => Ok(Tier::Pro),
            2 => Ok(Tier::Master),
            3 => Ok(Tier::Bundle),
            other => Err(TierError::UnknownCode(other)),
        }
    }

    /// Whether this tier is at least equivalent to `required` for the
    /// purpose of feature gating.
    pub fn satisfies(self, required: Tier) -> bool {
        // Bundle is treated as equivalent to Master for ranking purposes.
        let rank = |t: Tier| -> u8 {
            match t {
                Tier::Discovery => 0,
                Tier::Pro => 1,
                Tier::Master | Tier::Bundle => 2,
            }
        };
        rank(self) >= rank(required)
    }
}

/// Structured payload of a license key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LicensePayload {
    /// Format version (`1` for the current encoding).
    pub version: u8,
    /// Tier — also redundantly encoded in the human-readable prefix.
    pub tier: Tier,
    /// First 4 bytes of `SHA256(lowercase(email))` (cf. [`crate::email_hash`]).
    pub email_hash: u32,
    /// Days since 2026-01-01 at issuance time.
    pub issued_days: u16,
    /// Bitmask of fine-grained feature flags (reserved for future use).
    pub features: u16,
}

impl LicensePayload {
    /// Pack the payload into a `u128` such that the 72 payload bits occupy
    /// bit positions `[28..100)` (high 72 of the 100-bit license space).
    /// Bits `[0..28)` (the MAC zone) and `[100..128)` (unused) are zero.
    pub fn pack72(self) -> u128 {
        // Layout, most-significant first within the 72 payload bits:
        //   4  4   32         16            16
        // [ver][tier][email_hash][issued_days][features]
        let v = (self.version as u128) & 0xF;
        let t = (self.tier as u128) & 0xF;
        let e = self.email_hash as u128;
        let d = self.issued_days as u128;
        let f = self.features as u128;
        let low72 = (v << 68) | (t << 64) | (e << 32) | (d << 16) | f;
        // Shift left by MAC_BITS so the payload sits at bit positions [28..100).
        low72 << crate::MAC_BITS
    }

    /// Unpack a `u128` whose payload occupies bits `[28..100)` into a
    /// structured payload.
    pub fn unpack72(packed: u128) -> Result<Self, TierError> {
        let p = packed >> crate::MAC_BITS;
        let version = ((p >> 68) & 0xF) as u8;
        let tier_code = ((p >> 64) & 0xF) as u8;
        let email_hash = ((p >> 32) & 0xFFFF_FFFF) as u32;
        let issued_days = ((p >> 16) & 0xFFFF) as u16;
        let features = (p & 0xFFFF) as u16;
        Ok(LicensePayload {
            version,
            tier: Tier::from_code(tier_code)?,
            email_hash,
            issued_days,
            features,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_label_roundtrip() {
        for tier in [Tier::Discovery, Tier::Pro, Tier::Master, Tier::Bundle] {
            assert_eq!(Tier::from_label(tier.label()).unwrap(), tier);
        }
    }

    #[test]
    fn tier_code_roundtrip() {
        for tier in [Tier::Discovery, Tier::Pro, Tier::Master, Tier::Bundle] {
            assert_eq!(Tier::from_code(tier as u8).unwrap(), tier);
        }
    }

    #[test]
    fn tier_from_label_is_case_insensitive() {
        assert_eq!(Tier::from_label("deco").unwrap(), Tier::Discovery);
        assert_eq!(Tier::from_label("Pro").unwrap(), Tier::Pro);
    }

    #[test]
    fn tier_from_label_rejects_unknown() {
        assert!(matches!(
            Tier::from_label("LOL"),
            Err(TierError::UnknownLabel(_))
        ));
    }

    #[test]
    fn tier_satisfies_orders_correctly() {
        assert!(Tier::Pro.satisfies(Tier::Discovery));
        assert!(!Tier::Discovery.satisfies(Tier::Pro));
        assert!(Tier::Master.satisfies(Tier::Pro));
        assert!(Tier::Bundle.satisfies(Tier::Master));
    }

    #[test]
    fn payload_pack_unpack_roundtrip() {
        let p = LicensePayload {
            version: 1,
            tier: Tier::Pro,
            email_hash: 0xDEADBEEF,
            issued_days: 1234,
            features: 0xC0DE,
        };
        let packed = p.pack72();
        let unpacked = LicensePayload::unpack72(packed).unwrap();
        assert_eq!(p, unpacked);
        // The lowest MAC_BITS bits are zero (reserved for the MAC).
        assert_eq!(packed & ((1u128 << crate::MAC_BITS) - 1), 0);
        // Total fits within TOTAL_BITS bits.
        assert_eq!(packed >> crate::TOTAL_BITS, 0);
    }

    #[test]
    fn payload_max_values_fit() {
        let p = LicensePayload {
            version: 0xF,
            tier: Tier::Bundle,
            email_hash: u32::MAX,
            issued_days: u16::MAX,
            features: u16::MAX,
        };
        let packed = p.pack72();
        let unpacked = LicensePayload::unpack72(packed).unwrap();
        assert_eq!(p, unpacked);
    }

    #[test]
    fn payload_rejects_unknown_tier_code() {
        // Forge a packed value with an invalid tier code 0xF
        // (payload occupies bits [MAC_BITS..MAC_BITS+72), so we shift
        // the version+tier bits accordingly).
        let mac_bits = crate::MAC_BITS;
        let bad: u128 = ((1u128 << 68) | (0xFu128 << 64)) << mac_bits;
        assert!(matches!(
            LicensePayload::unpack72(bad),
            Err(TierError::UnknownCode(0xF))
        ));
    }
}
