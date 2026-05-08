//! Cosine similarity over `f32` slices.
//!
//! Returns 0.0 if either input vector is zero-length or all-zero, so callers
//! never produce `NaN` from a comparison. Inputs of differing length panic in
//! debug builds; in release we return 0.0 to avoid crashing on malformed data.

/// Cosine similarity, in `[-1.0, 1.0]`. Returns `0.0` for empty / zero vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "vectors must share the same dimension");
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0_f32;
    let mut na = 0.0_f32;
    let mut nb = 0.0_f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_vectors_score_one() {
        let v = vec![1.0_f32, 2.0, 3.0];
        let s = cosine_similarity(&v, &v);
        assert!((s - 1.0).abs() < 1e-6, "got {s}");
    }

    #[test]
    fn opposite_vectors_score_minus_one() {
        let a = vec![1.0_f32, 2.0, 3.0];
        let b = vec![-1.0_f32, -2.0, -3.0];
        let s = cosine_similarity(&a, &b);
        assert!((s + 1.0).abs() < 1e-6, "got {s}");
    }

    #[test]
    fn orthogonal_vectors_score_zero() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![0.0_f32, 1.0];
        let s = cosine_similarity(&a, &b);
        assert!(s.abs() < 1e-6, "got {s}");
    }

    #[test]
    fn zero_vector_returns_zero_not_nan() {
        let a = vec![0.0_f32, 0.0];
        let b = vec![1.0_f32, 1.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn empty_inputs_return_zero() {
        let empty: Vec<f32> = Vec::new();
        assert_eq!(cosine_similarity(&empty, &empty), 0.0);
    }
}
