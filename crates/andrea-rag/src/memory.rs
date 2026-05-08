//! In-memory [`crate::VectorStore`] backed by a `BTreeMap` keyed by
//! `Document::id`. Used for unit tests, the acceptance suite, and as a
//! fallback when `sqlite-vec` is unavailable.
//!
//! Search is brute-force `O(n)` cosine over all stored vectors. Fine for
//! the ~1500 chunks of the FPA référentiel; `sqlite-vec` is plugged in
//! in a later sub-step for production builds.

use async_trait::async_trait;
use std::collections::BTreeMap;
use tokio::sync::RwLock;

use crate::cosine::cosine_similarity;
use crate::types::{Document, Hit, StoreError, VectorStore};

/// Default embedding dimension matching `bge-m3` so production embeddings
/// drop in without resizing.
pub const DEFAULT_DIM: usize = 1024;

#[derive(Debug, Clone)]
struct Entry {
    document: Document,
    embedding: Vec<f32>,
}

/// Simple, thread-safe in-memory vector store.
#[derive(Debug)]
pub struct InMemoryVectorStore {
    dim: usize,
    inner: RwLock<BTreeMap<String, Entry>>,
}

impl InMemoryVectorStore {
    /// Build an empty store expecting `dim`-dimensional embeddings.
    pub fn new(dim: usize) -> Self {
        assert!(dim > 0, "dim must be > 0");
        Self {
            dim,
            inner: RwLock::new(BTreeMap::new()),
        }
    }

    /// Total number of stored documents.
    pub async fn count(&self) -> usize {
        self.inner.read().await.len()
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new(DEFAULT_DIM)
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn upsert(&self, document: Document, embedding: Vec<f32>) -> Result<(), StoreError> {
        if embedding.len() != self.dim {
            return Err(StoreError::DimensionMismatch {
                expected: self.dim,
                got: embedding.len(),
            });
        }
        let mut guard = self.inner.write().await;
        guard.insert(
            document.id.clone(),
            Entry {
                document,
                embedding,
            },
        );
        Ok(())
    }

    async fn search(&self, query_embedding: &[f32], top_k: usize) -> Result<Vec<Hit>, StoreError> {
        if query_embedding.len() != self.dim {
            return Err(StoreError::DimensionMismatch {
                expected: self.dim,
                got: query_embedding.len(),
            });
        }
        if top_k == 0 {
            return Ok(Vec::new());
        }
        let guard = self.inner.read().await;
        let mut scored: Vec<Hit> = guard
            .values()
            .map(|entry| Hit {
                document: entry.document.clone(),
                score: cosine_similarity(query_embedding, &entry.embedding),
            })
            .collect();
        // Descending by score; ties broken by stable id order.
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.document.id.cmp(&b.document.id))
        });
        scored.truncate(top_k);
        Ok(scored)
    }

    async fn len(&self) -> usize {
        self.inner.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(id: &str, text: &str) -> Document {
        Document {
            id: id.to_string(),
            source_doc: "TEST".to_string(),
            ccp: None,
            cp: None,
            section: None,
            citation: format!("TEST source {id}"),
            url: None,
            text: text.to_string(),
        }
    }

    #[tokio::test]
    async fn upsert_then_search_returns_inserted_doc() {
        let store = InMemoryVectorStore::new(3);
        let v = vec![1.0, 0.0, 0.0];
        store.upsert(doc("a", "alpha"), v.clone()).await.unwrap();
        let hits = store.search(&v, 1).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].document.id, "a");
        assert!(hits[0].score > 0.99);
    }

    #[tokio::test]
    async fn upsert_overwrites_existing_id() {
        let store = InMemoryVectorStore::new(3);
        store
            .upsert(doc("a", "v1"), vec![1.0, 0.0, 0.0])
            .await
            .unwrap();
        store
            .upsert(doc("a", "v2"), vec![0.0, 1.0, 0.0])
            .await
            .unwrap();
        assert_eq!(store.count().await, 1);
        let hits = store.search(&[0.0, 1.0, 0.0], 1).await.unwrap();
        assert_eq!(hits[0].document.text, "v2");
    }

    #[tokio::test]
    async fn search_returns_top_k_in_descending_score() {
        let store = InMemoryVectorStore::new(2);
        store.upsert(doc("a", "a"), vec![1.0, 0.0]).await.unwrap();
        store.upsert(doc("b", "b"), vec![0.7, 0.7]).await.unwrap();
        store.upsert(doc("c", "c"), vec![0.0, 1.0]).await.unwrap();
        let hits = store.search(&[1.0, 0.0], 3).await.unwrap();
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].document.id, "a");
        assert_eq!(hits[1].document.id, "b");
        assert_eq!(hits[2].document.id, "c");
    }

    #[tokio::test]
    async fn dimension_mismatch_is_surfaced() {
        let store = InMemoryVectorStore::new(3);
        let err = store
            .upsert(doc("a", "x"), vec![1.0, 0.0])
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            StoreError::DimensionMismatch {
                expected: 3,
                got: 2
            }
        ));
        let err = store.search(&[1.0, 0.0], 5).await.unwrap_err();
        assert!(matches!(err, StoreError::DimensionMismatch { .. }));
    }

    #[tokio::test]
    async fn top_k_zero_returns_empty() {
        let store = InMemoryVectorStore::new(2);
        store.upsert(doc("a", "x"), vec![1.0, 0.0]).await.unwrap();
        let hits = store.search(&[1.0, 0.0], 0).await.unwrap();
        assert!(hits.is_empty());
    }

    #[tokio::test]
    async fn upsert_many_inserts_all() {
        let store = InMemoryVectorStore::new(2);
        let items = vec![
            (doc("a", "x"), vec![1.0, 0.0]),
            (doc("b", "y"), vec![0.0, 1.0]),
        ];
        store.upsert_many(items).await.unwrap();
        assert_eq!(store.len().await, 2);
        assert!(!store.is_empty().await);
    }
}
