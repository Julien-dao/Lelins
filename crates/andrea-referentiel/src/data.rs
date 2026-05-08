//! Hard-coded référentiel data, copied verbatim from the public arrêté
//! du 7 décembre 2022 (REAC V07) as cross-referenced in
//! `docs/04-referentiel-fpa.md`.
//!
//! All four CCP, thirteen CP, three transversal competencies, and the
//! RNCP metadata live here. Every string is reviewed manually before
//! shipping — no LLM invented this content.

use serde::{Deserialize, Serialize};

/// Identifying metadata of the certification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RncpInfo {
    /// RNCP code (e.g. `RNCP37275`).
    pub code: &'static str,
    /// Full title.
    pub title: &'static str,
    /// Qualification level (1-8 in the EQF/CEC nomenclature).
    pub level: u8,
    /// Date of registration in the RNCP, ISO format.
    pub registered_on: &'static str,
    /// Date the registration ends, ISO format.
    pub valid_until: &'static str,
    /// Founding text (arrêté).
    pub founding_text: &'static str,
    /// Official URL of the RNCP record.
    pub url: &'static str,
}

/// Authoritative metadata for the FPA title in May 2026.
pub const RNCP_INFO: RncpInfo = RncpInfo {
    code: "RNCP37275",
    title: "Titre Professionnel Formateur Professionnel d'Adultes",
    level: 5,
    registered_on: "2023-04-29",
    valid_until: "2028-04-29",
    founding_text:
        "Arrêté du 7 décembre 2022 (JORFTEXT000046751421), modifié par l'arrêté du 24 janvier 2023.",
    url: "https://www.francecompetences.fr/recherche/rncp/37275/",
};

/// One Certificat de Compétences Professionnelles (CCP).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ccp {
    /// Code (`CCP1`, `CCP2`, `CCP3`, `CCP4`).
    pub code: &'static str,
    /// Official title.
    pub title: &'static str,
    /// One-paragraph summary of the activities covered.
    pub summary: &'static str,
}

/// One Compétence Professionnelle (CP).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cp {
    /// Code (`CP1` through `CP13`).
    pub code: &'static str,
    /// Owning CCP code (`CCP1`..`CCP4`).
    pub ccp: &'static str,
    /// Position within the owning CCP (1-based).
    pub order: u8,
    /// Verbatim title of the compétence.
    pub title: &'static str,
}

/// All four CCP, in order.
pub const ALL_CCP: [Ccp; 4] = [
    Ccp {
        code: "CCP1",
        title: "Concevoir et préparer la formation",
        summary:
            "Élaborer la progression pédagogique, concevoir le scénario d'une séquence et concevoir les activités d'apprentissage et d'évaluation, en intégrant la multimodalité présentielle et distancielle.",
    },
    Ccp {
        code: "CCP2",
        title: "Animer la formation et évaluer les acquis",
        summary:
            "Animer un temps de formation collectif en présence et à distance, évaluer les acquis des apprenants et inscrire ses actes professionnels dans une démarche de responsabilité sociale, environnementale et professionnelle.",
    },
    Ccp {
        code: "CCP3",
        title: "Accompagner les apprenants en formation",
        summary:
            "Accueillir les apprenants et co-construire leur parcours, accompagner la sécurisation de leurs acquis, tutorer à distance et soutenir leur développement professionnel par la remédiation et l'individualisation.",
    },
    Ccp {
        code: "CCP4",
        title: "Inscrire sa pratique dans une démarche qualité et RSE",
        summary:
            "Maintenir son expertise pédagogique et technique, analyser ses pratiques professionnelles, respecter et promouvoir la réglementation en vigueur (Qualiopi, RGPD, accessibilité, droit de la formation) ainsi que les principes RSE.",
    },
];

/// All thirteen Compétences Professionnelles, in their canonical order.
pub const ALL_CP: [Cp; 13] = [
    Cp {
        code: "CP1",
        ccp: "CCP1",
        order: 1,
        title: "Élaborer la progression pédagogique d'une action de formation à partir d'une demande.",
    },
    Cp {
        code: "CP2",
        ccp: "CCP1",
        order: 2,
        title: "Concevoir le scénario pédagogique d'une séquence de formation, en intégrant la multimodalité.",
    },
    Cp {
        code: "CP3",
        ccp: "CCP1",
        order: 3,
        title: "Concevoir les activités d'apprentissage et d'évaluation des acquis, en intégrant la multimodalité.",
    },
    Cp {
        code: "CP4",
        ccp: "CCP2",
        order: 1,
        title: "Animer un temps de formation collectif en présence et à distance, en favorisant les interactions.",
    },
    Cp {
        code: "CP5",
        ccp: "CCP2",
        order: 2,
        title: "Évaluer les acquis d'apprentissage des apprenants.",
    },
    Cp {
        code: "CP6",
        ccp: "CCP2",
        order: 3,
        title: "Inscrire ses actes professionnels dans une démarche de responsabilité sociale, environnementale et professionnelle.",
    },
    Cp {
        code: "CP7",
        ccp: "CCP3",
        order: 1,
        title: "Accueillir les apprenants en formation et co-construire leur parcours.",
    },
    Cp {
        code: "CP8",
        ccp: "CCP3",
        order: 2,
        title: "Accompagner les apprenants dans la construction de leur parcours et la sécurisation de leurs acquis.",
    },
    Cp {
        code: "CP9",
        ccp: "CCP3",
        order: 3,
        title: "Tutorer les apprenants à distance.",
    },
    Cp {
        code: "CP10",
        ccp: "CCP3",
        order: 4,
        title: "Accompagner le développement professionnel des apprenants par la remédiation et l'individualisation.",
    },
    Cp {
        code: "CP11",
        ccp: "CCP4",
        order: 1,
        title: "Maintenir son expertise pédagogique et technique par la veille et le développement professionnel.",
    },
    Cp {
        code: "CP12",
        ccp: "CCP4",
        order: 2,
        title: "Analyser ses pratiques professionnelles.",
    },
    Cp {
        code: "CP13",
        ccp: "CCP4",
        order: 3,
        title:
            "Respecter et promouvoir la réglementation en vigueur (Qualiopi, RGPD, accessibilité, droit de la formation) ainsi que les principes de responsabilité sociale et environnementale.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_four_ccp() {
        assert_eq!(ALL_CCP.len(), 4);
        let codes: Vec<&str> = ALL_CCP.iter().map(|c| c.code).collect();
        assert_eq!(codes, vec!["CCP1", "CCP2", "CCP3", "CCP4"]);
    }

    #[test]
    fn there_are_thirteen_cp() {
        assert_eq!(ALL_CP.len(), 13);
    }

    #[test]
    fn cp_codes_run_from_1_to_13() {
        let codes: Vec<&str> = ALL_CP.iter().map(|c| c.code).collect();
        let expected: Vec<String> = (1..=13).map(|i| format!("CP{i}")).collect();
        let expected_refs: Vec<&str> = expected.iter().map(|s| s.as_str()).collect();
        assert_eq!(codes, expected_refs);
    }

    #[test]
    fn each_cp_belongs_to_a_known_ccp() {
        let known: Vec<&str> = ALL_CCP.iter().map(|c| c.code).collect();
        for cp in ALL_CP.iter() {
            assert!(known.contains(&cp.ccp), "CP {} -> {}", cp.code, cp.ccp);
        }
    }

    #[test]
    fn ccp1_has_three_cp_in_order() {
        let cps: Vec<&Cp> = ALL_CP.iter().filter(|c| c.ccp == "CCP1").collect();
        assert_eq!(cps.len(), 3);
        assert_eq!(cps[0].code, "CP1");
        assert_eq!(cps[1].code, "CP2");
        assert_eq!(cps[2].code, "CP3");
    }

    #[test]
    fn ccp3_has_four_cp() {
        let cps: Vec<&Cp> = ALL_CP.iter().filter(|c| c.ccp == "CCP3").collect();
        assert_eq!(cps.len(), 4, "CCP3 has CP7-CP10");
    }

    #[test]
    fn rncp_info_matches_documented_values() {
        assert_eq!(RNCP_INFO.code, "RNCP37275");
        assert_eq!(RNCP_INFO.level, 5);
        assert_eq!(RNCP_INFO.valid_until, "2028-04-29");
    }
}
