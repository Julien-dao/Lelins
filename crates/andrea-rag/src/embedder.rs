//! Embedding provider abstraction and deterministic test impls.

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Errors raised by [`EmbeddingProvider`] impls.
#[derive(Debug, Error)]
pub enum EmbedError {
    /// Backend (HTTP, model) failure.
    #[error("embedding backend error: {0}")]
    Backend(String),
    /// Empty input — most embedders refuse it.
    #[error("input text is empty")]
    EmptyInput,
}

/// Embedding provider abstraction.
///
/// Implementations must produce L2-normalized vectors of constant
/// [`Self::dim`] dimension so cosine similarity reduces to a dot product.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Embed a single text. Returns an L2-normalized vector of length [`Self::dim`].
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError>;

    /// Output dimension (e.g. 1024 for `bge-m3`, 768 for `nomic-embed-text`).
    fn dim(&self) -> usize;

    /// Bulk embed. Default impl loops over [`Self::embed`]; backends with
    /// batch APIs should override.
    async fn embed_many(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbedError> {
        let mut out = Vec::with_capacity(texts.len());
        for t in texts {
            out.push(self.embed(t).await?);
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// HashEmbedder — deterministic, all-text-distinct, useful for sanity tests.
// ---------------------------------------------------------------------------

/// Deterministic embedder that hashes the entire input text into a vector.
///
/// Two distinct inputs produce uncorrelated vectors. Useful for tests that
/// check upsert / fetch / dimension-mismatch behaviour but not for tests
/// that need *semantic* similarity.
#[derive(Debug, Clone)]
pub struct HashEmbedder {
    dim: usize,
}

impl HashEmbedder {
    /// New embedder producing `dim`-dimensional vectors.
    pub fn new(dim: usize) -> Self {
        assert!(dim > 0, "dim must be > 0");
        Self { dim }
    }

    fn embed_sync(&self, text: &str) -> Vec<f32> {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let seed = hasher.finalize();
        // xorshift64 seeded with the first 8 bytes of the digest.
        let mut state = u64::from_le_bytes(seed[0..8].try_into().unwrap()).max(1);
        let mut out = Vec::with_capacity(self.dim);
        for _ in 0..self.dim {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            // Map u64 to [-1, 1] uniformly.
            let v = ((state as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32;
            out.push(v);
        }
        normalize_l2(&mut out);
        out
    }
}

#[async_trait]
impl EmbeddingProvider for HashEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        if text.is_empty() {
            return Err(EmbedError::EmptyInput);
        }
        Ok(self.embed_sync(text))
    }
    fn dim(&self) -> usize {
        self.dim
    }
}

// ---------------------------------------------------------------------------
// KeywordEmbedder — bag-of-words hashed into a fixed-dim vector.
// Provides crude semantic similarity that's useful for retrieval tests.
// ---------------------------------------------------------------------------

/// Deterministic, semantically meaningful embedder for tests.
///
/// Tokenizes on whitespace + punctuation, lowercases, drops stop words and
/// short tokens, then maps each remaining token to a position via FNV-1a
/// hashing. Two texts sharing tokens have cosine similarity > 0.
///
/// Not suitable for production — used in unit and acceptance tests only.
#[derive(Debug, Clone)]
pub struct KeywordEmbedder {
    dim: usize,
}

impl KeywordEmbedder {
    /// New embedder producing `dim`-dimensional bag-of-words vectors.
    pub fn new(dim: usize) -> Self {
        assert!(dim > 0, "dim must be > 0");
        Self { dim }
    }

    fn embed_sync(&self, text: &str) -> Vec<f32> {
        let mut v = vec![0.0_f32; self.dim];
        let mut total = 0_u32;
        for token in tokenize(text) {
            if STOP_WORDS_FR.contains(&token.as_str()) || token.len() < 3 {
                continue;
            }
            let h = fnv1a64(token.as_bytes());
            let idx = (h as usize) % self.dim;
            v[idx] += 1.0;
            total += 1;
        }
        if total > 0 {
            normalize_l2(&mut v);
        }
        v
    }
}

#[async_trait]
impl EmbeddingProvider for KeywordEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        if text.is_empty() {
            return Err(EmbedError::EmptyInput);
        }
        Ok(self.embed_sync(text))
    }
    fn dim(&self) -> usize {
        self.dim
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_lowercase().next().unwrap_or(c)
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

const STOP_WORDS_FR: &[&str] = &[
    "le", "la", "les", "un", "une", "des", "du", "de", "et", "ou", "à", "au", "aux", "ce", "ces",
    "cette", "cet", "qui", "que", "quoi", "dont", "où", "ne", "pas", "plus", "moins", "mais",
    "donc", "car", "ni", "or", "en", "dans", "sur", "sous", "par", "pour", "vers", "chez", "avec",
    "sans", "se", "sa", "son", "ses", "leur", "leurs", "tout", "tous", "toute", "toutes", "il",
    "elle", "ils", "elles", "on", "nous", "vous", "je", "tu", "moi", "toi", "lui", "eux", "est",
    "sont", "était", "été", "avoir", "être", "fait", "très", "tres", "aussi", "comme", "alors",
    "ainsi", "puis", "déjà", "deja", "encore", "afin", "lors", "lorsque", "quand", "si", "même",
    "meme", "tant", "y",
];

fn fnv1a64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn normalize_l2(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cosine::cosine_similarity;

    #[tokio::test]
    async fn hash_embedder_is_deterministic() {
        let e = HashEmbedder::new(64);
        let a = e.embed("hello").await.unwrap();
        let b = e.embed("hello").await.unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[tokio::test]
    async fn hash_embedder_outputs_are_normalized() {
        let e = HashEmbedder::new(64);
        let v = e.embed("anything").await.unwrap();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "got {norm}");
    }

    #[tokio::test]
    async fn keyword_embedder_picks_up_shared_tokens() {
        let e = KeywordEmbedder::new(256);
        let q = e.embed("la pédagogie active du formateur").await.unwrap();
        let close = e
            .embed("Marcel Lebrun et la pédagogie active des adultes")
            .await
            .unwrap();
        let far = e
            .embed("recette de la galette des rois traditionnelle")
            .await
            .unwrap();
        let s_close = cosine_similarity(&q, &close);
        let s_far = cosine_similarity(&q, &far);
        assert!(s_close > s_far, "close={s_close} far={s_far}");
        assert!(s_close > 0.3, "got {s_close}");
    }

    #[tokio::test]
    async fn keyword_embedder_drops_stop_words() {
        let e = KeywordEmbedder::new(256);
        let with_stops = e.embed("le formateur de la séance").await.unwrap();
        let without = e.embed("formateur séance").await.unwrap();
        let s = cosine_similarity(&with_stops, &without);
        assert!(s > 0.95, "got {s}");
    }

    #[tokio::test]
    async fn embedders_reject_empty_input() {
        let e = HashEmbedder::new(64);
        assert!(matches!(e.embed("").await, Err(EmbedError::EmptyInput)));
        let e = KeywordEmbedder::new(64);
        assert!(matches!(e.embed("").await, Err(EmbedError::EmptyInput)));
    }

    #[tokio::test]
    async fn embed_many_preserves_order() {
        let e = HashEmbedder::new(32);
        let out = e.embed_many(&["alpha", "beta", "alpha"]).await.unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(out[0], out[2]);
        assert_ne!(out[0], out[1]);
    }
}
