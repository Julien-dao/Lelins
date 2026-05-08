//! Build the initial RAG corpus from the structured référentiel data.
//!
//! Produces one [`Document`] per CCP, one per CP, and several for
//! cross-cutting topics (épreuves, DP, jury, voies d'accès, métadonnées
//! RNCP). All chunks share the same `source_doc` and emit the canonical
//! citation format expected by the system prompt.

use andrea_rag::Document;

use crate::data::{ALL_CCP, ALL_CP, RNCP_INFO};

/// Version label embedded in every citation. Bumped whenever the underlying
/// référentiel V07 is replaced (e.g. when a new arrêté is published).
pub const REFERENTIEL_VERSION: &str = "REAC V07 21/12/2022";

/// Stable URL for the official France Compétences record.
const REFERENTIEL_URL: &str = "https://www.francecompetences.fr/recherche/rncp/37275/";

/// Number of chunks emitted by [`build_initial_chunks`]. Asserted by tests
/// so we notice if the corpus shape ever changes silently.
pub const INITIAL_CHUNK_COUNT: usize = 26;

/// Build the bootstrap corpus.
pub fn build_initial_chunks() -> Vec<Document> {
    let mut docs = Vec::with_capacity(INITIAL_CHUNK_COUNT);

    docs.push(rncp_overview());
    for ccp in ALL_CCP.iter() {
        docs.push(ccp_overview(ccp));
    }
    for cp in ALL_CP.iter() {
        docs.push(cp_chunk(cp));
    }
    docs.push(epreuve_msp());
    docs.push(epreuve_entretien_technique());
    docs.push(epreuve_entretien_final());
    docs.push(epreuve_questionnement_dp());
    docs.push(jury_composition());
    docs.push(dossier_professionnel());
    docs.push(voies_acces());
    docs.push(transversal_competencies());

    debug_assert_eq!(docs.len(), INITIAL_CHUNK_COUNT);
    docs
}

fn citation_for_section(section: &str) -> String {
    format!("{REFERENTIEL_VERSION}, {section}")
}

fn citation_for_cp(cp_code: &str, ccp_code: &str) -> String {
    format!("{REFERENTIEL_VERSION}, {ccp_code}, {cp_code}")
}

fn rncp_overview() -> Document {
    let info = RNCP_INFO;
    let text = format!(
        "Le titre professionnel « {title} » est enregistré au Répertoire national des certifications \
professionnelles (RNCP) sous le numéro {code}, au niveau de qualification {level} (équivalent Bac+2 dans \
la nomenclature européenne EQF/CEC). Il est certifié par le Ministère du Travail. \
Date d'enregistrement : {registered}. Date de fin de validité : {valid_until}. \
Texte fondateur : {founding}.",
        title = info.title,
        code = info.code,
        level = info.level,
        registered = info.registered_on,
        valid_until = info.valid_until,
        founding = info.founding_text,
    );
    Document {
        id: "rncp-overview".to_string(),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: None,
        cp: None,
        section: Some("Identification RNCP".to_string()),
        citation: citation_for_section("RNCP37275, fiche officielle"),
        url: Some(info.url.to_string()),
        text,
    }
}

fn ccp_overview(ccp: &crate::data::Ccp) -> Document {
    let text = format!(
        "{code} — {title}. {summary}",
        code = ccp.code,
        title = ccp.title,
        summary = ccp.summary,
    );
    Document {
        id: format!("{}-overview", ccp.code.to_lowercase()),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: Some(ccp.code.to_string()),
        cp: None,
        section: Some("Présentation du CCP".to_string()),
        citation: citation_for_section(&format!("{} ({})", ccp.code, ccp.title)),
        url: Some(REFERENTIEL_URL.to_string()),
        text,
    }
}

fn cp_chunk(cp: &crate::data::Cp) -> Document {
    let ccp = ALL_CCP
        .iter()
        .find(|c| c.code == cp.ccp)
        .expect("data integrity");
    let text = format!(
        "{cp_code} (rattachée à {ccp_code} — {ccp_title}) : {title}",
        cp_code = cp.code,
        ccp_code = cp.ccp,
        ccp_title = ccp.title,
        title = cp.title,
    );
    Document {
        id: format!("{}-titre", cp.code.to_lowercase()),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: Some(cp.ccp.to_string()),
        cp: Some(cp.code.to_string()),
        section: Some("Intitulé de la compétence".to_string()),
        citation: citation_for_cp(cp.code, cp.ccp),
        url: Some(REFERENTIEL_URL.to_string()),
        text,
    }
}

fn epreuve_msp() -> Document {
    Document {
        id: "epreuve-msp".to_string(),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: None,
        cp: None,
        section: Some("Épreuve — Mise en Situation Professionnelle".to_string()),
        citation: citation_for_section("Épreuve MSP"),
        url: Some("https://www.jurytitreprofessionnel.fr/".to_string()),
        text: "Mise en Situation Professionnelle (MSP) reconstituée. Le candidat tire au sort un sujet \
parmi six. Préparation hors présence du jury : 45 minutes. Présentation orale devant le jury : \
10 minutes. L'épreuve démontre la maîtrise des gestes professionnels du métier de formateur sur les \
quatre activités-types du référentiel."
            .to_string(),
    }
}

fn epreuve_entretien_technique() -> Document {
    Document {
        id: "epreuve-entretien-technique".to_string(),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: None,
        cp: None,
        section: Some("Épreuve — Entretien technique".to_string()),
        citation: citation_for_section("Entretien technique"),
        url: Some("https://www.jurytitreprofessionnel.fr/".to_string()),
        text:
            "Entretien technique de 20 minutes mené par le jury. Il approfondit les compétences \
techniques liées au sujet de la MSP et investigue les zones non couvertes par cette dernière. \
L'entretien doit attester de la profondeur d'expertise et de la rigueur méthodologique du candidat."
                .to_string(),
    }
}

fn epreuve_entretien_final() -> Document {
    Document {
        id: "epreuve-entretien-final".to_string(),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: None,
        cp: None,
        section: Some("Épreuve — Entretien final".to_string()),
        citation: citation_for_section("Entretien final"),
        url: Some("https://www.jurytitreprofessionnel.fr/".to_string()),
        text: "Entretien final de 20 minutes. Il porte sur le positionnement professionnel du candidat, \
sa projection dans le métier, sa représentation de l'emploi et de son environnement, et notamment sur \
les dimensions éthique et de responsabilité sociale, environnementale et professionnelle (RSE)."
            .to_string(),
    }
}

fn epreuve_questionnement_dp() -> Document {
    Document {
        id: "epreuve-questionnement-dp".to_string(),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: None,
        cp: None,
        section: Some("Épreuve — Questionnement à partir des productions et du DP".to_string()),
        citation: citation_for_section("Questionnement DP"),
        url: Some("https://www.jurytitreprofessionnel.fr/".to_string()),
        text: "Questionnement à partir des productions et du Dossier Professionnel. Durée indicative : \
1 h 45 (cumulé avec la présentation des activités-types et la discussion autour du DP). Le jury croise \
les déclarations du candidat avec les preuves consignées dans le DP pour valider l'authenticité et \
l'analyse réflexive."
            .to_string(),
    }
}

fn jury_composition() -> Document {
    Document {
        id: "jury-composition".to_string(),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: None,
        cp: None,
        section: Some("Composition du jury".to_string()),
        citation: citation_for_section("Composition du jury"),
        url: Some("https://www.jurytitreprofessionnel.fr/".to_string()),
        text: "Le jury est composé de deux professionnels du métier (formateurs d'adultes en exercice ou \
ayant exercé depuis moins de 5 ans). Les jurés sont habilités par le Préfet via la DREETS / DDETS, \
après vérification d'au moins 3 ans d'expérience dans le métier visé. L'habilitation est valable \
jusqu'à la fin de la durée de validité du titre. Cadre juridique : arrêté du 22/12/2015, articles 5 à 8."
            .to_string(),
    }
}

fn dossier_professionnel() -> Document {
    Document {
        id: "dossier-professionnel".to_string(),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: None,
        cp: None,
        section: Some("Dossier Professionnel (DP)".to_string()),
        citation: citation_for_section("Dossier Professionnel"),
        url: Some("https://www.jurytitreprofessionnel.fr/".to_string()),
        text: "Le Dossier Professionnel est le support central de l'épreuve. Il comprend une page de \
garde, un sommaire, une description de la pratique professionnelle organisée par activité-type \
(soit quatre sections, une par CCP), et une déclaration sur l'honneur signée. Règle des exemples : \
trois exemples maximum par activité-type, trois pages maximum par exemple, soit jusqu'à 12 exemples \
sur environ 36 pages utiles. Le jury évalue la précision, l'analyse réflexive, la cohérence avec les \
compétences visées et l'authenticité des situations décrites."
            .to_string(),
    }
}

fn voies_acces() -> Document {
    Document {
        id: "voies-acces".to_string(),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: None,
        cp: None,
        section: Some("Voies d'accès".to_string()),
        citation: citation_for_section("Voies d'accès"),
        url: Some(REFERENTIEL_URL.to_string()),
        text: "Le titre est accessible par cinq voies : formation continue (parcours majoritaire), \
formation initiale (rare), apprentissage / contrat de professionnalisation, validation des acquis \
de l'expérience (VAE) via France VAE, ou validation par blocs avec capitalisation des CCP pendant \
cinq ans. Durée typique d'un parcours complet : 600 à 1 000 heures, dont environ 315 heures de stage \
en entreprise. Calendrier moyen : 6 à 12 mois selon le rythme."
            .to_string(),
    }
}

fn transversal_competencies() -> Document {
    Document {
        id: "competences-transversales".to_string(),
        source_doc: REFERENTIEL_VERSION.to_string(),
        ccp: None,
        cp: None,
        section: Some("Compétences transversales".to_string()),
        citation: citation_for_section("Compétences transversales"),
        url: Some(REFERENTIEL_URL.to_string()),
        text: "Trois compétences transversales sont mobilisées dans toutes les activités du formateur : \
mobiliser un environnement numérique professionnel, communiquer à l'écrit comme à l'oral en \
s'adaptant aux interlocuteurs, agir dans une logique inclusive (handicap, illettrisme, mixité)."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_initial_chunks_returns_documented_count() {
        let chunks = build_initial_chunks();
        assert_eq!(chunks.len(), INITIAL_CHUNK_COUNT);
    }

    #[test]
    fn every_chunk_has_a_citation_with_the_referentiel_version() {
        let chunks = build_initial_chunks();
        for c in chunks {
            assert!(
                c.citation.contains("REAC V07 21/12/2022"),
                "missing version in citation: {}",
                c.citation
            );
            assert!(!c.text.trim().is_empty(), "empty text on chunk {}", c.id);
            assert_eq!(c.source_doc, REFERENTIEL_VERSION);
        }
    }

    #[test]
    fn cp_chunks_carry_both_ccp_and_cp_metadata() {
        let chunks = build_initial_chunks();
        let cp_chunks: Vec<_> = chunks.iter().filter(|c| c.cp.is_some()).collect();
        assert_eq!(cp_chunks.len(), 13);
        for c in cp_chunks {
            assert!(c.ccp.is_some());
            assert!(c.citation.contains(c.cp.as_ref().unwrap()));
            assert!(c.citation.contains(c.ccp.as_ref().unwrap()));
        }
    }

    #[test]
    fn ccp_chunks_have_no_cp() {
        let chunks = build_initial_chunks();
        let ccp_chunks: Vec<_> = chunks
            .iter()
            .filter(|c| c.ccp.is_some() && c.cp.is_none())
            .collect();
        assert_eq!(ccp_chunks.len(), 4);
    }

    #[test]
    fn ids_are_unique() {
        let chunks = build_initial_chunks();
        let mut ids: Vec<_> = chunks.iter().map(|c| c.id.clone()).collect();
        ids.sort();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "duplicate id detected");
    }

    #[test]
    fn epreuve_chunks_are_present() {
        let chunks = build_initial_chunks();
        let ids: Vec<&str> = chunks.iter().map(|c| c.id.as_str()).collect();
        for needed in [
            "epreuve-msp",
            "epreuve-entretien-technique",
            "epreuve-entretien-final",
            "epreuve-questionnement-dp",
            "jury-composition",
            "dossier-professionnel",
            "voies-acces",
            "competences-transversales",
        ] {
            assert!(ids.contains(&needed), "missing chunk {needed}");
        }
    }
}
