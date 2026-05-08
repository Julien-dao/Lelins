//! [`Retriever`] augments each turn's system prompt with relevant FPA
//! référentiel chunks.
//!
//! Two implementations live here:
//!
//! - [`NoopRetriever`] — returns an empty block; used in tests and as the
//!   default until the corpus is ingested.
//! - [`RagRetriever`] — wraps an [`andrea_rag::EmbeddingProvider`] and an
//!   [`andrea_rag::VectorStore`], performs a semantic search per turn, and
//!   formats the top-`k` hits using
//!   [`andrea_rag::retrieve::format_for_prompt`].

use async_trait::async_trait;
use std::sync::Arc;

use andrea_rag::retrieve::{format_for_prompt, search_filtered};
use andrea_rag::{EmbeddingProvider, VectorStore};

/// Retrieval contract consumed by [`crate::ConversationEngine`].
///
/// Implementations must never panic and should swallow transient backend
/// errors silently — a missing context block is preferable to a failed
/// turn. The returned string is inserted verbatim into the system prompt
/// under the `== EXTRAITS DU RÉFÉRENTIEL ==` section.
#[async_trait]
pub trait Retriever: Send + Sync {
    /// Build a context block for this query. Empty string means "no
    /// context" (the prompt will fall back to its standard "no chunk
    /// found" message).
    async fn augment(&self, query: &str) -> String;
}

/// Pass-through retriever that always returns an empty block.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopRetriever;

#[async_trait]
impl Retriever for NoopRetriever {
    async fn augment(&self, _query: &str) -> String {
        String::new()
    }
}

/// Production retriever backed by an embedding provider and a vector store.
pub struct RagRetriever {
    embedder: Arc<dyn EmbeddingProvider>,
    store: Arc<dyn VectorStore>,
    /// Top-k hits to keep.
    pub top_k: usize,
    /// Cosine similarity floor; hits below this are discarded so the prompt
    /// is not polluted with off-topic chunks.
    pub min_score: f32,
}

impl RagRetriever {
    /// Build a retriever with `top_k = 4` and `min_score = 0.15`, the
    /// defaults validated by the acceptance suite.
    pub fn new(embedder: Arc<dyn EmbeddingProvider>, store: Arc<dyn VectorStore>) -> Self {
        Self {
            embedder,
            store,
            top_k: 4,
            min_score: 0.15,
        }
    }

    /// Override the score threshold.
    pub fn with_min_score(mut self, min_score: f32) -> Self {
        self.min_score = min_score;
        self
    }

    /// Override the top-k.
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }
}

#[async_trait]
impl Retriever for RagRetriever {
    async fn augment(&self, query: &str) -> String {
        match search_filtered(
            &*self.embedder,
            &*self.store,
            query,
            self.top_k,
            self.min_score,
        )
        .await
        {
            Ok(hits) => format_for_prompt(&hits),
            Err(e) => {
                tracing::warn!(error = %e, "RAG retrieval failed, returning empty context");
                String::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use andrea_rag::ingest::ingest_documents;
    use andrea_rag::{Document, InMemoryVectorStore, KeywordEmbedder};

    fn doc(id: &str, text: &str) -> Document {
        Document {
            id: id.to_string(),
            source_doc: "TEST".to_string(),
            ccp: None,
            cp: None,
            section: None,
            citation: format!("TEST {id}"),
            url: None,
            text: text.to_string(),
        }
    }

    #[tokio::test]
    async fn noop_returns_empty_string() {
        let r = NoopRetriever;
        assert_eq!(r.augment("anything").await, "");
    }

    async fn build_retriever() -> RagRetriever {
        let embedder = Arc::new(KeywordEmbedder::new(256));
        let store = Arc::new(InMemoryVectorStore::new(256));
        let docs = vec![
            doc("a", "tutorat à distance pour les apprenants"),
            doc("b", "concevoir le scénario pédagogique en multimodalité"),
            doc("c", "recette de la galette des rois"),
        ];
        let embedder_for_ingest: &KeywordEmbedder = &embedder;
        let store_for_ingest: &InMemoryVectorStore = &store;
        ingest_documents(embedder_for_ingest, store_for_ingest, docs, |_, _| {})
            .await
            .unwrap();
        RagRetriever::new(embedder, store)
    }

    #[tokio::test]
    async fn rag_retriever_returns_relevant_block() {
        let r = build_retriever().await;
        let block = r.augment("tutorat distance apprenants").await;
        assert!(block.contains("tutorat à distance"), "got: {block}");
        assert!(block.contains("Source : TEST"));
    }

    #[tokio::test]
    async fn rag_retriever_returns_empty_for_off_topic_query() {
        let r = build_retriever().await.with_min_score(0.5);
        let block = r.augment("calcul intégrale différentielle").await;
        assert_eq!(block, "");
    }
}
