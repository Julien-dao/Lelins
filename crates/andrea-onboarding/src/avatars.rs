//! Curated 12-portrait avatar gallery.
//!
//! Images are bundled separately in `apps/desktop/public/avatars/<id>.jpg`
//! (sourced manually per `docs/02-direction-artistique.md`); this module
//! holds only the metadata so the desktop app can present the gallery,
//! filter by origin, and remember the chosen avatar after restart.
//!
//! Diversity targets (per `docs/02`) :
//! - 6 femmes / 6 hommes
//! - tranche d'âge 25-65
//! - origines : européen·ne, maghrébin·e, afro-caribéen·ne, asiatique,
//!   réunionnais·e (créole), mahorais·e, métis·se.

use serde::{Deserialize, Serialize};

/// Apparent gender of the portrait (used to pick the right `accord_genre`
/// in the rendered ANDREA Formateur v1 prompt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AvatarGender {
    /// Apparent female.
    Female,
    /// Apparent male.
    Male,
}

impl AvatarGender {
    /// French phrasing inserted into `{{ accord_genre }}`.
    pub fn accord_genre(self) -> &'static str {
        match self {
            AvatarGender::Female => "votre formatrice",
            AvatarGender::Male => "votre formateur",
        }
    }
}

/// Apparent origin (declarative — we never infer it at runtime). Used to
/// let users in DOM-TOM filter the gallery if they want, while keeping
/// the default selection diverse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvatarOrigin {
    /// European-presenting.
    European,
    /// Maghrebi-presenting.
    Maghrebi,
    /// Afro-Caribbean-presenting.
    AfroCaribbean,
    /// Asian-presenting.
    Asian,
    /// Réunionnais·e / créole-presenting (priority for La Réunion users).
    Creole,
    /// Mahorais·e-presenting.
    Mahorais,
    /// Mixed / métis·se.
    Mixed,
}

/// One option in the avatar gallery.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AvatarOption {
    /// Stable identifier (filename stem under `public/avatars/`).
    pub id: &'static str,
    /// Display label shown next to the portrait.
    pub display_name: &'static str,
    /// Apparent gender, drives the prompt's `accord_genre`.
    pub gender: AvatarGender,
    /// Apparent origin, drives the gallery filter.
    pub origin: AvatarOrigin,
    /// Rough apparent age band ("25-35", "60-65", …).
    pub age_band: &'static str,
}

/// Twelve curated portraits. The order is the default display order.
pub const AVATAR_GALLERY: [AvatarOption; 12] = [
    AvatarOption {
        id: "femme-creole-45",
        display_name: "Marie",
        gender: AvatarGender::Female,
        origin: AvatarOrigin::Creole,
        age_band: "40-50",
    },
    AvatarOption {
        id: "homme-creole-50",
        display_name: "Jean-Marc",
        gender: AvatarGender::Male,
        origin: AvatarOrigin::Creole,
        age_band: "45-55",
    },
    AvatarOption {
        id: "femme-mahoraise-35",
        display_name: "Salama",
        gender: AvatarGender::Female,
        origin: AvatarOrigin::Mahorais,
        age_band: "30-40",
    },
    AvatarOption {
        id: "femme-maghrebine-40",
        display_name: "Yasmine",
        gender: AvatarGender::Female,
        origin: AvatarOrigin::Maghrebi,
        age_band: "35-45",
    },
    AvatarOption {
        id: "homme-maghrebin-55",
        display_name: "Karim",
        gender: AvatarGender::Male,
        origin: AvatarOrigin::Maghrebi,
        age_band: "50-60",
    },
    AvatarOption {
        id: "femme-afro-caribeenne-30",
        display_name: "Naïma",
        gender: AvatarGender::Female,
        origin: AvatarOrigin::AfroCaribbean,
        age_band: "25-35",
    },
    AvatarOption {
        id: "homme-afro-caribeen-45",
        display_name: "Tony",
        gender: AvatarGender::Male,
        origin: AvatarOrigin::AfroCaribbean,
        age_band: "40-50",
    },
    AvatarOption {
        id: "femme-asiatique-35",
        display_name: "Linh",
        gender: AvatarGender::Female,
        origin: AvatarOrigin::Asian,
        age_band: "30-40",
    },
    AvatarOption {
        id: "femme-europeenne-50",
        display_name: "Anne",
        gender: AvatarGender::Female,
        origin: AvatarOrigin::European,
        age_band: "45-55",
    },
    AvatarOption {
        id: "homme-europeen-60",
        display_name: "Bernard",
        gender: AvatarGender::Male,
        origin: AvatarOrigin::European,
        age_band: "55-65",
    },
    AvatarOption {
        id: "homme-metis-30",
        display_name: "Mathieu",
        gender: AvatarGender::Male,
        origin: AvatarOrigin::Mixed,
        age_band: "25-35",
    },
    AvatarOption {
        id: "homme-asiatique-40",
        display_name: "Hugo",
        gender: AvatarGender::Male,
        origin: AvatarOrigin::Asian,
        age_band: "35-45",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_has_twelve_entries() {
        assert_eq!(AVATAR_GALLERY.len(), 12);
    }

    #[test]
    fn ids_are_unique() {
        let mut ids: Vec<&str> = AVATAR_GALLERY.iter().map(|a| a.id).collect();
        ids.sort();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n);
    }

    #[test]
    fn gender_split_is_balanced() {
        let females = AVATAR_GALLERY
            .iter()
            .filter(|a| a.gender == AvatarGender::Female)
            .count();
        let males = AVATAR_GALLERY.len() - females;
        assert!(
            (females as i32 - males as i32).abs() <= 2,
            "imbalanced: {females} F vs {males} M"
        );
    }

    #[test]
    fn ultramarine_representation_is_present() {
        let creole = AVATAR_GALLERY
            .iter()
            .filter(|a| a.origin == AvatarOrigin::Creole)
            .count();
        let mahorais = AVATAR_GALLERY
            .iter()
            .filter(|a| a.origin == AvatarOrigin::Mahorais)
            .count();
        assert!(
            creole >= 2,
            "need at least 2 créole portraits (got {creole})"
        );
        assert!(
            mahorais >= 1,
            "need at least 1 Mahorais portrait (got {mahorais})"
        );
    }

    #[test]
    fn accord_genre_is_french() {
        assert_eq!(AvatarGender::Female.accord_genre(), "votre formatrice");
        assert_eq!(AvatarGender::Male.accord_genre(), "votre formateur");
    }
}
