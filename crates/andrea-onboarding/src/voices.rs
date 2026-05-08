//! Catalogue of Piper TTS voices proposed during onboarding.
//!
//! Per `docs/02-direction-artistique.md` and `docs/05-stack-technique.md`,
//! the v1 default lineup is `siwis-medium` (FR neutre féminin) and
//! `tom-medium` (FR neutre masculin), with two extra voices behind a
//! "Plus de choix" toggle. All voices ship under the OHF-Voice/piper1-gpl
//! fork (GPL-3.0 in sidecar mode — see `docs/08-decisions.md` decision #4).

use serde::{Deserialize, Serialize};

/// Apparent gender of the voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoiceGender {
    /// Female-presenting voice.
    Female,
    /// Male-presenting voice.
    Male,
}

/// One Piper voice option presented to the user.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VoiceOption {
    /// Stable identifier used by the desktop app to look up the model file.
    pub id: &'static str,
    /// Display label.
    pub display_name: &'static str,
    /// Piper model identifier (matches the .onnx filename stem).
    pub piper_model_id: &'static str,
    /// Apparent gender.
    pub gender: VoiceGender,
    /// Sample rate the voice produces, in hertz.
    pub sample_rate: u32,
    /// One-line description shown next to the play button.
    pub blurb: &'static str,
    /// Whether this voice is part of the default lineup or only shown
    /// behind "Plus de choix".
    pub default_pick: bool,
}

/// Voice catalogue.
pub const VOICE_CATALOGUE: [VoiceOption; 4] = [
    VoiceOption {
        id: "siwis",
        display_name: "Siwis (féminin, neutre)",
        piper_model_id: "fr_FR-siwis-medium",
        gender: VoiceGender::Female,
        sample_rate: 22_050,
        blurb: "Voix de référence ANDREA, claire et posée — recommandée pour les sessions longues.",
        default_pick: true,
    },
    VoiceOption {
        id: "tom",
        display_name: "Tom (masculin, neutre)",
        piper_model_id: "fr_FR-tom-medium",
        gender: VoiceGender::Male,
        sample_rate: 22_050,
        blurb: "Voix masculine équilibrée, bonne intelligibilité en mode vocal.",
        default_pick: true,
    },
    VoiceOption {
        id: "mls-1840",
        display_name: "MLS 1840 (féminin, alternative)",
        piper_model_id: "fr_FR-mls_1840-medium",
        gender: VoiceGender::Female,
        sample_rate: 22_050,
        blurb: "Variante féminine plus chaleureuse — testez à l'écoute pour décider.",
        default_pick: false,
    },
    VoiceOption {
        id: "gilles",
        display_name: "Gilles (masculin, alternative)",
        piper_model_id: "fr_FR-gilles-low",
        gender: VoiceGender::Male,
        sample_rate: 16_000,
        blurb: "Variante masculine légère, qualité audio plus basse mais latence très faible.",
        default_pick: false,
    },
];

/// Quick lookup helper used by the desktop app and the Tauri commands.
pub fn find_voice(id: &str) -> Option<&'static VoiceOption> {
    VOICE_CATALOGUE.iter().find(|v| v.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_size_matches_doc() {
        assert_eq!(VOICE_CATALOGUE.len(), 4);
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<&str> = VOICE_CATALOGUE.iter().map(|v| v.id).collect();
        ids.sort();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n);
    }

    #[test]
    fn default_picks_cover_both_genders() {
        let defaults: Vec<_> = VOICE_CATALOGUE.iter().filter(|v| v.default_pick).collect();
        assert_eq!(defaults.len(), 2);
        assert!(defaults.iter().any(|v| v.gender == VoiceGender::Female));
        assert!(defaults.iter().any(|v| v.gender == VoiceGender::Male));
    }

    #[test]
    fn find_voice_returns_known_id() {
        assert_eq!(
            find_voice("siwis").unwrap().display_name,
            "Siwis (féminin, neutre)"
        );
        assert!(find_voice("nonexistent").is_none());
    }

    #[test]
    fn piper_model_ids_use_fr_locale() {
        for v in VOICE_CATALOGUE.iter() {
            assert!(
                v.piper_model_id.starts_with("fr_FR-"),
                "voice {} has non-FR locale: {}",
                v.id,
                v.piper_model_id
            );
        }
    }
}
