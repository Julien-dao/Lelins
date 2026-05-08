//! Acceptance set used to validate the RAG pipeline against the bootstrap
//! corpus.
//!
//! Each [`EvalQuestion`] is a paraphrased question from a real apprenant
//! plus the set of chunk IDs that **must** appear in the top-3 hits. The
//! integration test in `tests/eval_acceptance.rs` runs every question and
//! enforces a minimum recall rate ([`ACCEPTANCE_RECALL`]).
//!
//! When the official PDFs are integrated in a follow-up step, this set
//! grows to ~30 questions with finer-grained gold IDs. Today it covers the
//! 26 chunks of the bootstrap corpus.

/// Fraction of [`ANDREA_EVAL_SET`] questions whose top-3 must contain at
/// least one expected chunk for the test to pass.
pub const ACCEPTANCE_RECALL: f32 = 0.90;

/// One acceptance question.
#[derive(Debug, Clone, Copy)]
pub struct EvalQuestion {
    /// Stable identifier for human reporting.
    pub id: &'static str,
    /// Free-form query an apprenant might ask in French.
    pub query: &'static str,
    /// Chunk IDs (cf. `chunks.rs`) the retriever must surface.
    pub expected_chunk_ids: &'static [&'static str],
    /// Free-form note explaining why we expect those chunks.
    pub rationale: &'static str,
}

/// 30 acceptance questions covering the bootstrap corpus.
pub const ANDREA_EVAL_SET: [EvalQuestion; 30] = [
    // ---- CCP overviews (4) ----
    EvalQuestion {
        id: "Q01",
        query: "Que recouvre le CCP1 sur la conception de la formation ?",
        expected_chunk_ids: &["ccp1-overview", "cp1-titre", "cp2-titre", "cp3-titre"],
        rationale: "CCP1 — Concevoir et préparer la formation",
    },
    EvalQuestion {
        id: "Q02",
        query: "Quelle activité couvre le CCP2 d'animation ?",
        expected_chunk_ids: &["ccp2-overview", "cp4-titre", "cp5-titre"],
        rationale: "CCP2 — Animer la formation et évaluer les acquis",
    },
    EvalQuestion {
        id: "Q03",
        query: "L'accompagnement des apprenants relève de quel CCP ?",
        expected_chunk_ids: &["ccp3-overview", "cp7-titre", "cp8-titre"],
        rationale: "CCP3 — Accompagner les apprenants en formation",
    },
    EvalQuestion {
        id: "Q04",
        query: "Démarche qualité, Qualiopi, RSE et veille pédagogique ?",
        expected_chunk_ids: &["ccp4-overview", "cp11-titre", "cp12-titre", "cp13-titre"],
        rationale: "CCP4 — Inscrire sa pratique dans une démarche qualité et RSE",
    },
    // ---- CP titles (13) ----
    EvalQuestion {
        id: "Q05",
        query: "Comment élaborer la progression pédagogique d'une action de formation ?",
        expected_chunk_ids: &["cp1-titre", "ccp1-overview"],
        rationale: "CP1",
    },
    EvalQuestion {
        id: "Q06",
        query: "Concevoir le scénario pédagogique d'une séquence en intégrant la multimodalité",
        expected_chunk_ids: &["cp2-titre"],
        rationale: "CP2",
    },
    EvalQuestion {
        id: "Q07",
        query: "Concevoir les activités d'apprentissage et d'évaluation des acquis",
        expected_chunk_ids: &["cp3-titre"],
        rationale: "CP3",
    },
    EvalQuestion {
        id: "Q08",
        query: "Animer un temps de formation collectif en présence et à distance",
        expected_chunk_ids: &["cp4-titre"],
        rationale: "CP4",
    },
    EvalQuestion {
        id: "Q09",
        query: "Évaluer les acquis d'apprentissage des apprenants",
        expected_chunk_ids: &["cp5-titre"],
        rationale: "CP5",
    },
    EvalQuestion {
        id: "Q10",
        query: "Inscrire ses actes professionnels dans une démarche de responsabilité sociale",
        expected_chunk_ids: &["cp6-titre"],
        rationale: "CP6 — RSE",
    },
    EvalQuestion {
        id: "Q11",
        query: "Accueillir les apprenants en formation et co-construire leur parcours",
        expected_chunk_ids: &["cp7-titre"],
        rationale: "CP7",
    },
    EvalQuestion {
        id: "Q12",
        query: "Sécuriser les acquis et accompagner la construction du parcours",
        expected_chunk_ids: &["cp8-titre"],
        rationale: "CP8",
    },
    EvalQuestion {
        id: "Q13",
        query: "Tutorer les apprenants à distance",
        expected_chunk_ids: &["cp9-titre"],
        rationale: "CP9 — tutorat distanciel",
    },
    EvalQuestion {
        id: "Q14",
        query:
            "Remédiation et individualisation pour le développement professionnel des apprenants",
        expected_chunk_ids: &["cp10-titre"],
        rationale: "CP10",
    },
    EvalQuestion {
        id: "Q15",
        query:
            "Maintenir son expertise pédagogique par la veille et le développement professionnel",
        expected_chunk_ids: &["cp11-titre"],
        rationale: "CP11",
    },
    EvalQuestion {
        id: "Q16",
        query: "Analyser ses pratiques professionnelles",
        expected_chunk_ids: &["cp12-titre"],
        rationale: "CP12",
    },
    EvalQuestion {
        id: "Q17",
        query: "Respecter Qualiopi RGPD accessibilité droit de la formation",
        expected_chunk_ids: &["cp13-titre", "ccp4-overview"],
        rationale: "CP13",
    },
    // ---- Épreuves (4) ----
    EvalQuestion {
        id: "Q18",
        query: "Combien de minutes pour la mise en situation professionnelle ?",
        expected_chunk_ids: &["epreuve-msp"],
        rationale: "MSP",
    },
    EvalQuestion {
        id: "Q19",
        query: "En quoi consiste l'entretien technique du jury ?",
        expected_chunk_ids: &["epreuve-entretien-technique"],
        rationale: "Entretien technique",
    },
    EvalQuestion {
        id: "Q20",
        query: "Comment se déroule l'entretien final avec le jury ?",
        expected_chunk_ids: &["epreuve-entretien-final"],
        rationale: "Entretien final",
    },
    EvalQuestion {
        id: "Q21",
        query: "Questionnement à partir des productions et du dossier professionnel",
        expected_chunk_ids: &["epreuve-questionnement-dp"],
        rationale: "Questionnement DP",
    },
    // ---- DP, jury, accès, transversal (6) ----
    EvalQuestion {
        id: "Q22",
        query: "Combien d'exemples de pratique professionnelle dans le DP ?",
        expected_chunk_ids: &["dossier-professionnel"],
        rationale: "DP — règle des 3 exemples",
    },
    EvalQuestion {
        id: "Q23",
        query: "Combien de personnes composent le jury et qui les habilite ?",
        expected_chunk_ids: &["jury-composition"],
        rationale: "Jury",
    },
    EvalQuestion {
        id: "Q24",
        query: "Quelles voies d'accès au titre, VAE comprise ?",
        expected_chunk_ids: &["voies-acces"],
        rationale: "Voies d'accès",
    },
    EvalQuestion {
        id: "Q25",
        query: "Compétences transversales : numérique, écrit oral, inclusion",
        expected_chunk_ids: &["competences-transversales"],
        rationale: "Transversal",
    },
    EvalQuestion {
        id: "Q26",
        query: "Quel est le numéro RNCP du titre FPA ?",
        expected_chunk_ids: &["rncp-overview"],
        rationale: "RNCP37275",
    },
    EvalQuestion {
        id: "Q27",
        query: "Le titre FPA est-il enregistré jusqu'à quelle date ?",
        expected_chunk_ids: &["rncp-overview"],
        rationale: "Validité 2028-04-29",
    },
    // ---- Cas concrets / synonymes (3) ----
    EvalQuestion {
        id: "Q28",
        query: "Comment intégrer la multimodalité dans une séquence ?",
        expected_chunk_ids: &["cp2-titre", "cp3-titre"],
        rationale: "Multimodalité — CP2 et CP3",
    },
    EvalQuestion {
        id: "Q29",
        query: "Hybride présence distance pour favoriser les interactions",
        expected_chunk_ids: &["cp4-titre"],
        rationale: "CP4 hybridation animation",
    },
    EvalQuestion {
        id: "Q30",
        query: "Durée typique de la formation et stage en entreprise",
        expected_chunk_ids: &["voies-acces"],
        rationale: "Durée 600-1000h, stage 315h",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_set_has_expected_size() {
        assert_eq!(ANDREA_EVAL_SET.len(), 30);
    }

    #[test]
    fn eval_question_ids_are_unique() {
        let mut ids: Vec<&str> = ANDREA_EVAL_SET.iter().map(|q| q.id).collect();
        ids.sort();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n);
    }

    #[test]
    fn each_question_has_at_least_one_expected_chunk() {
        for q in ANDREA_EVAL_SET.iter() {
            assert!(
                !q.expected_chunk_ids.is_empty(),
                "{}: no expected chunk",
                q.id
            );
        }
    }

    #[test]
    fn every_expected_chunk_id_exists_in_corpus() {
        let chunks = crate::build_initial_chunks();
        let known: Vec<&str> = chunks.iter().map(|c| c.id.as_str()).collect();
        for q in ANDREA_EVAL_SET.iter() {
            for expected in q.expected_chunk_ids.iter() {
                assert!(
                    known.contains(expected),
                    "{}: expected chunk `{}` not in corpus",
                    q.id,
                    expected
                );
            }
        }
    }
}
