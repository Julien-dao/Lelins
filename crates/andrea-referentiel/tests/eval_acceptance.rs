//! Acceptance test for the bootstrap RAG pipeline.
//!
//! Builds the initial corpus, ingests it through the in-memory store with
//! the deterministic [`KeywordEmbedder`], runs every question in
//! [`ANDREA_EVAL_SET`], and asserts that the recall rate (top-3 contains at
//! least one expected chunk) meets [`ACCEPTANCE_RECALL`].
//!
//! When this test fails, run with `--nocapture` — the failures are printed
//! one line each so you can see exactly which questions regressed.

use andrea_rag::{
    ingest::ingest_documents, retrieve::search, EmbeddingProvider, InMemoryVectorStore,
    KeywordEmbedder, VectorStore,
};
use andrea_referentiel::{build_initial_chunks, ACCEPTANCE_RECALL, ANDREA_EVAL_SET};

const EMBEDDING_DIM: usize = 1024;
const TOP_K: usize = 3;

#[tokio::test]
async fn rag_recall_meets_acceptance_threshold() {
    let embedder = KeywordEmbedder::new(EMBEDDING_DIM);
    let store = InMemoryVectorStore::new(EMBEDDING_DIM);
    let chunks = build_initial_chunks();
    let summary = ingest_documents(&embedder, &store, chunks, |_, _| {})
        .await
        .expect("ingestion succeeds");
    assert!(summary.written > 0);
    assert_eq!(summary.skipped_empty, 0);

    let embedder_dyn: &dyn EmbeddingProvider = &embedder;
    let store_dyn: &dyn VectorStore = &store;

    let mut hits_total = 0;
    let mut failures = Vec::new();

    for q in ANDREA_EVAL_SET.iter() {
        let results = search(embedder_dyn, store_dyn, q.query, TOP_K)
            .await
            .expect("retrieval succeeds");
        let returned_ids: Vec<&str> = results.iter().map(|r| r.document.id.as_str()).collect();
        let any_match = q
            .expected_chunk_ids
            .iter()
            .any(|expected| returned_ids.contains(expected));
        if any_match {
            hits_total += 1;
        } else {
            failures.push((q.id, q.query, returned_ids.join(", ")));
        }
    }

    let recall = hits_total as f32 / ANDREA_EVAL_SET.len() as f32;
    if recall < ACCEPTANCE_RECALL {
        eprintln!(
            "RAG acceptance failure: {hits_total}/{total} = {recall:.2} < threshold {threshold:.2}",
            total = ANDREA_EVAL_SET.len(),
            recall = recall,
            threshold = ACCEPTANCE_RECALL,
        );
        for (id, query, returned) in &failures {
            eprintln!("  {id}  query=\"{query}\"  top-3=[{returned}]");
        }
        panic!("recall {recall:.2} below threshold {ACCEPTANCE_RECALL:.2}");
    }
}

#[tokio::test]
async fn citations_include_canonical_version_string() {
    let embedder = KeywordEmbedder::new(EMBEDDING_DIM);
    let store = InMemoryVectorStore::new(EMBEDDING_DIM);
    ingest_documents(&embedder, &store, build_initial_chunks(), |_, _| {})
        .await
        .unwrap();
    let hits = search(&embedder, &store, "mise en situation professionnelle", 3)
        .await
        .unwrap();
    assert!(!hits.is_empty());
    for h in hits {
        assert!(
            h.document.citation.contains("REAC V07 21/12/2022"),
            "missing canonical version in citation: {}",
            h.document.citation
        );
    }
}

#[tokio::test]
async fn idempotent_reingestion_does_not_grow_store() {
    let embedder = KeywordEmbedder::new(EMBEDDING_DIM);
    let store = InMemoryVectorStore::new(EMBEDDING_DIM);
    ingest_documents(&embedder, &store, build_initial_chunks(), |_, _| {})
        .await
        .unwrap();
    let n1 = store.len().await;
    ingest_documents(&embedder, &store, build_initial_chunks(), |_, _| {})
        .await
        .unwrap();
    let n2 = store.len().await;
    assert_eq!(n1, n2);
}
