//! Retrieval pipeline: embed the query, ask the store for the top-k hits,
//! and format ANDREA-style citation blocks for the system prompt.

use thiserror::Error;

use crate::embedder::EmbedError;
use crate::types::{Hit, StoreError, VectorStore};
use crate::EmbeddingProvider;

/// Errors raised by the retrieval pipeline.
#[derive(Debug, Error)]
pub enum RetrieveError {
    /// Embedding step failed.
    #[error(transparent)]
    Embed(#[from] EmbedError),
    /// Vector store step failed.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// Empty query.
    #[error("query text is empty")]
    EmptyQuery,
}

/// Embed `query` and return the `top_k` nearest documents.
pub async fn search(
    embedder: &dyn EmbeddingProvider,
    store: &dyn VectorStore,
    query: &str,
    top_k: usize,
) -> Result<Vec<Hit>, RetrieveError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(RetrieveError::EmptyQuery);
    }
    let embedding = embedder.embed(trimmed).await?;
    let hits = store.search(&embedding, top_k).await?;
    Ok(hits)
}

/// Same as [`search`] but drops hits whose score is below `min_score`.
/// Useful to avoid feeding clearly off-topic chunks into the prompt.
pub async fn search_filtered(
    embedder: &dyn EmbeddingProvider,
    store: &dyn VectorStore,
    query: &str,
    top_k: usize,
    min_score: f32,
) -> Result<Vec<Hit>, RetrieveError> {
    let mut hits = search(embedder, store, query, top_k).await?;
    hits.retain(|h| h.score >= min_score);
    Ok(hits)
}

/// Render a list of hits as a block ready to be substituted into the
/// `{{ retrieved_chunks }}` placeholder of the ANDREA system prompt.
///
/// Each hit becomes a numbered paragraph:
///
/// ```text
/// [1] Source : REAC V07 21/12/2022, CCP1, CP3 (score 0.83)
///     Concevoir les activités d'apprentissage et d'évaluation des acquis…
/// ```
///
/// Returns the empty string if `hits` is empty so the caller can
/// distinguish "no relevant chunk" and emit the appropriate fallback line.
pub fn format_for_prompt(hits: &[Hit]) -> String {
    if hits.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for (i, hit) in hits.iter().enumerate() {
        out.push_str(&format!(
            "[{n}] Source : {citation} (score {score:.2})\n    {text}\n\n",
            n = i + 1,
            citation = hit.document.citation,
            score = hit.score,
            text = compact_one_line(&hit.document.text, 600),
        ));
    }
    // Trim the trailing blank line.
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

fn compact_one_line(text: &str, max_chars: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= max_chars {
        collapsed
    } else {
        let mut out: String = collapsed.chars().take(max_chars).collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ingest::ingest_documents;
    use crate::types::Document;
    use crate::{HashEmbedder, InMemoryVectorStore, KeywordEmbedder};

    fn doc(id: &str, ccp: &str, cp: &str, text: &str) -> Document {
        Document {
            id: id.to_string(),
            source_doc: "REAC V07 21/12/2022".to_string(),
            ccp: Some(ccp.to_string()),
            cp: Some(cp.to_string()),
            section: Some("Test".to_string()),
            citation: format!("REAC V07 21/12/2022, {ccp}, {cp}"),
            url: None,
            text: text.to_string(),
        }
    }

    async fn build_index() -> (KeywordEmbedder, InMemoryVectorStore) {
        let embedder = KeywordEmbedder::new(512);
        let store = InMemoryVectorStore::new(512);
        let docs = vec![
            doc(
                "cp3",
                "CCP1",
                "CP3",
                "Concevoir les activités d'apprentissage et d'évaluation des acquis, en intégrant la multimodalité.",
            ),
            doc(
                "cp4",
                "CCP2",
                "CP4",
                "Animer un temps de formation collectif en présence et à distance, en favorisant les interactions.",
            ),
            doc(
                "cp9",
                "CCP3",
                "CP9",
                "Tutorer les apprenants à distance pour sécuriser leur progression.",
            ),
            doc(
                "msp",
                "ÉPREUVES",
                "MSP",
                "La Mise en Situation Professionnelle dure 45 minutes de préparation puis 10 minutes de présentation devant le jury.",
            ),
        ];
        ingest_documents(&embedder, &store, docs, |_, _| {})
            .await
            .unwrap();
        (embedder, store)
    }

    #[tokio::test]
    async fn keyword_query_retrieves_topical_chunk_first() {
        let (embedder, store) = build_index().await;
        let hits = search(&embedder, &store, "tutorat à distance", 3)
            .await
            .unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].document.id, "cp9", "got {:?}", hits[0].document.id);
    }

    #[tokio::test]
    async fn empty_query_is_rejected() {
        let embedder = HashEmbedder::new(32);
        let store = InMemoryVectorStore::new(32);
        let err = search(&embedder, &store, "   ", 3).await.unwrap_err();
        assert!(matches!(err, RetrieveError::EmptyQuery));
    }

    #[tokio::test]
    async fn search_filtered_drops_low_score_hits() {
        let (embedder, store) = build_index().await;
        let strict = search_filtered(&embedder, &store, "concevoir activités évaluation", 4, 0.2)
            .await
            .unwrap();
        assert!(strict.iter().all(|h| h.score >= 0.2));
        assert!(!strict.is_empty());

        let nothing = search_filtered(&embedder, &store, "concevoir activités évaluation", 4, 1.5)
            .await
            .unwrap();
        assert!(nothing.is_empty());
    }

    #[tokio::test]
    async fn format_for_prompt_renders_numbered_block() {
        let (embedder, store) = build_index().await;
        let hits = search(&embedder, &store, "mise en situation", 2)
            .await
            .unwrap();
        let block = format_for_prompt(&hits);
        assert!(block.starts_with("[1] Source : "));
        assert!(block.contains("REAC V07 21/12/2022"));
        // Second hit appears too.
        assert!(block.contains("[2] Source : "));
    }

    #[tokio::test]
    async fn format_for_prompt_handles_empty_hits() {
        assert_eq!(format_for_prompt(&[]), "");
    }

    #[test]
    fn compact_one_line_caps_long_text() {
        let long = "a".repeat(1000);
        let out = compact_one_line(&long, 50);
        assert!(out.chars().count() <= 51, "{}", out.chars().count());
        assert!(out.ends_with('…'));
    }
}
