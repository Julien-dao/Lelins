//! Variables interpolated into the ANDREA Formateur v1 system prompt.

use serde::{Deserialize, Serialize};

/// Snapshot of the apprenant's profile + session state, ready to feed into
/// [`crate::render_formateur_v1`].
///
/// Field naming matches the `{{ var }}` placeholders in
/// `packages/prompts/andrea-formateur-v1.md`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileVars {
    /// Display name of the formateur as customised by the user. Default: `ANDREA`.
    pub formateur_nom: String,
    /// Apprenant's first name as captured during onboarding.
    pub prenom: String,
    /// How the apprenant wants to be addressed by name (`Marie`, `Mme Dupont`, …).
    pub appel: String,
    /// Form of address: `tu` or `vous`. ANDREA defaults to vouvoiement
    /// (per `docs/08-decisions.md` decision #6 — choice always presented
    /// to the user, no silent default).
    pub tu_vous: String,
    /// Gendered phrasing for the formateur (`votre formatrice` / `votre
    /// formateur`), derived from the chosen avatar.
    pub accord_genre: String,
    /// Apprenant's training context (e.g. `apprenant FPA en CFA`).
    pub contexte: String,
    /// Starting level (e.g. `formatrice en exercice depuis 5 ans`).
    pub niveau_depart: String,
    /// Stated objective (e.g. `titre complet`, `CCP1 seul`).
    pub objectif: String,
    /// Preferred pace (e.g. `régulier, 3 h par semaine`).
    pub cadence: String,
    /// Exam date if known, else a friendly fallback like `non fixée`.
    pub date_epreuve: String,
    /// One-paragraph summary of progression by competence (computed from
    /// the `progression` SQLite table).
    pub progression_summary: String,
    /// Summary of the previous session, or a placeholder for the first one.
    pub session_summary: String,
}

impl ProfileVars {
    /// Sensible defaults for a brand-new install (used by tests and the
    /// first-launch placeholder before onboarding completes).
    pub fn placeholder() -> Self {
        Self {
            formateur_nom: "ANDREA".to_string(),
            prenom: "apprenant·e".to_string(),
            appel: "vous".to_string(),
            tu_vous: "vous".to_string(),
            accord_genre: "votre formatrice".to_string(),
            contexte: "non précisé".to_string(),
            niveau_depart: "non précisé".to_string(),
            objectif: "non précisé".to_string(),
            cadence: "non précisée".to_string(),
            date_epreuve: "non fixée".to_string(),
            progression_summary: "aucune compétence travaillée pour l'instant".to_string(),
            session_summary: "première session — pas d'historique antérieur.".to_string(),
        }
    }
}

impl Default for ProfileVars {
    fn default() -> Self {
        Self::placeholder()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_has_french_fallbacks() {
        let p = ProfileVars::placeholder();
        assert_eq!(p.formateur_nom, "ANDREA");
        assert_eq!(p.tu_vous, "vous");
        assert!(p.session_summary.contains("première session"));
    }

    #[test]
    fn profile_serializes_round_trip() {
        let p = ProfileVars::placeholder();
        let json = serde_json::to_string(&p).unwrap();
        let back: ProfileVars = serde_json::from_str(&json).unwrap();
        assert_eq!(back.formateur_nom, "ANDREA");
    }
}
