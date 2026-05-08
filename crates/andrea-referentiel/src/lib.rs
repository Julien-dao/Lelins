//! Structured Titre Pro FPA référentiel data, ready to feed into the
//! RAG pipeline.
//!
//! Source of truth: `docs/04-referentiel-fpa.md`, itself derived from
//! the REAC V07 of 21 December 2022 (RNCP n°37275). The data here is the
//! **bootstrapped corpus** that ANDREA ships with; the full REAC and RE
//! PDFs are layered on top in a later step once the manual download is
//! validated against the official sources.
//!
//! All citations follow the canonical format mandated by the system
//! prompt: `Source : REAC V07 21/12/2022, CCPx, CPy`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod chunks;
mod data;
mod eval;

pub use chunks::{build_initial_chunks, INITIAL_CHUNK_COUNT, REFERENTIEL_VERSION};
pub use data::{Ccp, Cp, RncpInfo, ALL_CCP, ALL_CP, RNCP_INFO};
pub use eval::{EvalQuestion, ACCEPTANCE_RECALL, ANDREA_EVAL_SET};

/// Re-export of the underlying RAG document type so consumers don't need
/// to depend on `andrea-rag` directly.
pub use andrea_rag::Document;
