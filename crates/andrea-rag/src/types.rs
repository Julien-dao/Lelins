//! Document, hit, and vector store types.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A single chunk of source material to be indexed and retrieved.
///
/// Field naming matches `docs/04-referentiel-fpa.md`'s chunking strategy:
/// every chunk carries enough metadata to render a citation back to the
/// user (`Source : REAC V07 21/12/2022, CCP1, CP3`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    /// Stable, unique identifier (e.g. `reac-v07-cp3-savoirs`).
    pub id: String,
    /// Reference document (e.g. `REAC V07 21/12/2022`).
    pub source_doc: String,
    /// CCP this chunk belongs to, when applicable (e.g. `CCP1`).
    pub ccp: Option<String>,
    /// Compétence professionnelle (e.g. `CP3`), when applicable.
    pub cp: Option<String>,
    /// Sub-section within the CP (e.g. `Savoirs associés`).
    pub section: Option<String>,
    /// Pre-rendered citation displayed to the user.
    pub citation: String,
    /// Stable URL pointing back to the official source.
    pub url: Option<String>,
    /// The actual text content (chunked appropriately for the embedding
    /// model's context window, typically 200-500 tokens).
    pub text: String,
}

/// A retrieval result.
#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    /// The matched document.
    pub document: Document,
    /// Cosine similarity to the query, in `[-1.0, 1.0]`.
    pub score: f32,
}

/// Errors raised by [`VectorStore`] implementations.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Embedding dimension does not match the store's expectation.
    #[error("embedding dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch {
        /// Expected dimension.
        expected: usize,
        /// Actual dimension of the offending input.
        got: usize,
    },
    /// Backend-specific failure.
    #[error("vector store backend error: {0}")]
    Backend(String),
}

/// Storage abstraction for embeddings.
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Insert (or update) a single document. Idempotent on `document.id`.
    async fn upsert(&self, document: Document, embedding: Vec<f32>) -> Result<(), StoreError>;

    /// Bulk variant for ingestion pipelines. Default implementation falls
    /// back to repeated `upsert`.
    async fn upsert_many(&self, items: Vec<(Document, Vec<f32>)>) -> Result<(), StoreError> {
        for (doc, emb) in items {
            self.upsert(doc, emb).await?;
        }
        Ok(())
    }

    /// Return the `top_k` nearest neighbours of `query_embedding`,
    /// in descending similarity order.
    async fn search(&self, query_embedding: &[f32], top_k: usize) -> Result<Vec<Hit>, StoreError>;

    /// Total number of documents stored.
    async fn len(&self) -> usize;

    /// Whether the store contains zero documents.
    async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}
