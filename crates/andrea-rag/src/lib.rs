//! Retrieval-augmented generation primitives for ANDREA.
//!
//! Two abstractions, three concrete pieces:
//!
//! - [`EmbeddingProvider`] — turns text into a dense vector. The production
//!   impl wraps `andrea_llm::LlmProvider::embed` (bge-m3 by default); tests
//!   use [`KeywordEmbedder`] (deterministic bag-of-words) or
//!   [`HashEmbedder`] (deterministic but uncorrelated).
//! - [`VectorStore`] — stores `(id, embedding, document)` triples and finds
//!   the top-`k` nearest neighbours. The production impl will plug into
//!   `sqlite-vec` via `tauri-plugin-rusqlite2`; tests use
//!   [`InMemoryVectorStore`].
//!
//! Pipelines:
//!
//! - [`ingest::ingest_documents`] — for each input document, computes the
//!   embedding and inserts it. Idempotent: calling twice with the same
//!   `id` overwrites the previous entry.
//! - [`retrieve::search`] — embeds the query, runs the store's nearest
//!   neighbour search, and formats human-readable citations.
//!
//! All trait methods are async so a future `sqlite-vec` impl can offload
//! work to `tokio::task::spawn_blocking`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod cosine;
mod embedder;
pub mod ingest;
mod memory;
pub mod retrieve;
mod types;

pub use cosine::cosine_similarity;
pub use embedder::{EmbedError, EmbeddingProvider, HashEmbedder, KeywordEmbedder};
pub use memory::InMemoryVectorStore;
pub use types::{Document, Hit, StoreError, VectorStore};
