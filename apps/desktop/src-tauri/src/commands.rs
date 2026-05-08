//! Tauri commands invoked by the React frontend.
//!
//! Each command is `async` to avoid blocking the runtime and returns a
//! serializable result. Domain errors are converted to strings here for
//! easy consumption from JS — finer error types can be reintroduced later
//! once the frontend has a proper error UI.

use andrea_license::{verify, LicenseError, Tier};
use serde::Serialize;

/// Trivial health-check used by the frontend at startup.
#[tauri::command]
pub async fn ping() -> &'static str {
    "pong"
}

/// Result returned by [`license_validate`].
#[derive(Debug, Clone, Serialize)]
pub struct LicenseSummary {
    pub canonical: String,
    pub tier: &'static str,
    pub version: u8,
    pub issued_days: u16,
    pub features: u16,
}

impl From<andrea_license::License> for LicenseSummary {
    fn from(l: andrea_license::License) -> Self {
        Self {
            canonical: l.canonical,
            tier: l.payload.tier.label(),
            version: l.payload.version,
            issued_days: l.payload.issued_days,
            features: l.payload.features,
        }
    }
}

/// Validate a user-typed key against the embedded server secret and the
/// installed user's email. Returns a `LicenseSummary` on success, or a
/// human-readable error message on failure.
///
/// In v1 the server secret is embedded at compile time. The blocklist will
/// be loaded from a bundled JSON file in a later step.
#[tauri::command]
pub async fn license_validate(raw_key: String, email: String) -> Result<LicenseSummary, String> {
    let secret = option_env!("ANDREA_LICENSE_SECRET")
        .map(|s| s.as_bytes().to_vec())
        .unwrap_or_else(|| b"andrea-dev-placeholder-secret-do-not-ship".to_vec());
    let revocation: &[&str] = &[];
    verify(&raw_key, &email, &secret, revocation)
        .map(LicenseSummary::from)
        .map_err(license_error_message)
}

/// Convenience: parse a key without verifying its MAC, useful for showing
/// the tier badge in the UI before the user has confirmed their email.
#[tauri::command]
pub async fn license_info(raw_key: String) -> Result<&'static str, String> {
    let normalized = andrea_license::normalize(&raw_key);
    if !normalized.starts_with("ANDREA") {
        return Err("La clé doit commencer par ANDREA.".to_string());
    }
    let after_brand = &normalized[6..];
    for tier_len in [4usize, 3] {
        if after_brand.len() >= tier_len {
            let label = &after_brand[..tier_len];
            if let Ok(tier) = Tier::from_label(label) {
                return Ok(tier.label());
            }
        }
    }
    Err("Tier inconnu.".to_string())
}

fn license_error_message(err: LicenseError) -> String {
    use LicenseError::*;
    match err {
        Format(_) => "Format de clé invalide. Vérifiez la saisie.".to_string(),
        Crockford(_) => "Caractère invalide dans la clé.".to_string(),
        Tier(_) => "Tier de licence non reconnu.".to_string(),
        TierMismatch { .. } => "Le tier indiqué ne correspond pas à la clé.".to_string(),
        UnsupportedVersion(v) => format!(
            "Cette version de clé ({v}) n'est pas reconnue. Mettez à jour ANDREA."
        ),
        BadMac => "La signature de la clé est invalide.".to_string(),
        EmailMismatch => "Cette clé ne correspond pas à l'email saisi.".to_string(),
        Revoked => "Cette clé a été révoquée. Contactez le support.".to_string(),
    }
}
