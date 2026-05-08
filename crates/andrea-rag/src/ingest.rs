//! Ingestion pipeline: embed each document and upsert it into the store.

use thiserror::Error;

use crate::embedder::EmbedError;
use crate::types::{Document, StoreError, VectorStore};
use crate::EmbeddingProvider;

/// Errors raised by the ingestion pipeline.
#[derive(Debug, Error)]
pub enum IngestError {
    /// Embedding step failed.
    #[error(transparent)]
    Embed(#[from] EmbedError),
    /// Vector store step failed.
    #[error(transparent)]
    Store(#[from] StoreError),
}

/// Outcome of a single ingestion run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IngestSummary {
    /// Number of documents inserted or replaced.
    pub written: usize,
    /// Number of documents skipped because their `text` was empty.
    pub skipped_empty: usize,
}

/// Embed and upsert each document into `store`.
///
/// Calls to [`EmbeddingProvider::embed`] are sequential — for the FPA
/// référentiel (~30 docs in v1, ~1500 once expanded) this is fast enough
/// (Ollama bge-m3 averages 30-50 ms per chunk on Apple Silicon).
///
/// Documents whose `text` field is empty are skipped silently so the
/// caller can include placeholder rows in the corpus without breaking
/// ingestion.
///
/// `progress` is invoked once per document with `(current, total)` so the
/// UI can render a progress bar; pass `|_,_| {}` to ignore.
pub async fn ingest_documents(
    embedder: &dyn EmbeddingProvider,
    store: &dyn VectorStore,
    documents: Vec<Document>,
    mut progress: impl FnMut(usize, usize),
) -> Result<IngestSummary, IngestError> {
    let total = documents.len();
    let mut written = 0;
    let mut skipped_empty = 0;
    for (i, doc) in documents.into_iter().enumerate() {
        progress(i, total);
        let text = doc.text.trim();
        if text.is_empty() {
            skipped_empty += 1;
            continue;
        }
        let embedding = embedder.embed(text).await?;
        store.upsert(doc, embedding).await?;
        written += 1;
    }
    progress(total, total);
    Ok(IngestSummary {
        written,
        skipped_empty,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HashEmbedder, InMemoryVectorStore};

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
    async fn ingest_writes_all_docs() {
        let embedder = HashEmbedder::new(64);
        let store = InMemoryVectorStore::new(64);
        let docs = vec![doc("a", "alpha"), doc("b", "beta"), doc("c", "gamma")];
        let summary = ingest_documents(&embedder, &store, docs, |_, _| {})
            .await
            .unwrap();
        assert_eq!(summary.written, 3);
        assert_eq!(summary.skipped_empty, 0);
        assert_eq!(store.len().await, 3);
    }

    #[tokio::test]
    async fn ingest_skips_empty_text() {
        let embedder = HashEmbedder::new(32);
        let store = InMemoryVectorStore::new(32);
        let docs = vec![doc("a", "ok"), doc("b", "   "), doc("c", "ok2")];
        let summary = ingest_documents(&embedder, &store, docs, |_, _| {})
            .await
            .unwrap();
        assert_eq!(summary.written, 2);
        assert_eq!(summary.skipped_empty, 1);
        assert_eq!(store.len().await, 2);
    }

    #[tokio::test]
    async fn ingest_is_idempotent() {
        let embedder = HashEmbedder::new(32);
        let store = InMemoryVectorStore::new(32);
        let docs = vec![doc("a", "alpha"), doc("b", "beta")];
        ingest_documents(&embedder, &store, docs.clone(), |_, _| {})
            .await
            .unwrap();
        // Re-ingest with the same ids — store size stays the same.
        ingest_documents(&embedder, &store, docs, |_, _| {})
            .await
            .unwrap();
        assert_eq!(store.len().await, 2);
    }

    #[tokio::test]
    async fn ingest_progress_callback_observes_each_step() {
        let embedder = HashEmbedder::new(16);
        let store = InMemoryVectorStore::new(16);
        let docs = vec![doc("a", "x"), doc("b", "y"), doc("c", "z")];
        let mut events: Vec<(usize, usize)> = Vec::new();
        ingest_documents(&embedder, &store, docs, |cur, tot| {
            events.push((cur, tot));
        })
        .await
        .unwrap();
        assert_eq!(events.first(), Some(&(0, 3)));
        assert_eq!(events.last(), Some(&(3, 3)));
    }
}
