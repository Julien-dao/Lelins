//! Speech-to-text abstraction for ANDREA.
//!
//! All audio capture happens upstream in the desktop layer (cpal) and
//! arrives here as **16 kHz mono PCM** in [`i16`] samples — the canonical
//! format for whisper.cpp and most other STT engines.
//!
//! The [`Transcriber`] trait hides the engine. v1 ships a [`MockTranscriber`]
//! for tests and orchestration; the real Whisper implementation will be
//! plugged in behind the `whisper-cpp` feature once we build on macOS.
//!
//! # PCM convention
//!
//! - sample rate : 16 000 Hz
//! - channels    : 1 (mono)
//! - sample type : `i16` little-endian
//!
//! Callers feeding higher-rate audio must downsample (e.g. with `dasp` or
//! `rubato`) before calling [`Transcriber::transcribe`].

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod mock;
mod types;

pub use mock::MockTranscriber;
pub use types::{Segment, TranscribeError, TranscribeOptions, Transcript};

use async_trait::async_trait;

/// Canonical sample rate accepted by every [`Transcriber`].
pub const SAMPLE_RATE_HZ: u32 = 16_000;

/// Speech-to-text engine abstraction.
#[async_trait]
pub trait Transcriber: Send + Sync {
    /// Transcribe a slice of 16 kHz mono `i16` samples to text.
    async fn transcribe(
        &self,
        samples: &[i16],
        options: &TranscribeOptions,
    ) -> Result<Transcript, TranscribeError>;
}

/// Convert `i16` PCM in `[-32768, 32767]` to normalized `f32` in `[-1.0, 1.0]`.
///
/// Whisper.cpp consumes f32 internally; this helper isolates the conversion
/// so engines that already speak f32 (or future GPU paths) can skip it.
pub fn pcm_i16_to_f32(samples: &[i16]) -> Vec<f32> {
    // 32768.0 is the canonical full-scale divisor; using it avoids the off-by-one
    // overflow that 32767.0 introduces for the negative extreme.
    samples.iter().map(|&s| s as f32 / 32768.0).collect()
}

/// Estimate the duration of a 16 kHz mono PCM buffer in seconds.
pub fn duration_seconds(samples: &[i16]) -> f32 {
    samples.len() as f32 / SAMPLE_RATE_HZ as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pcm_conversion_handles_extremes() {
        let v = pcm_i16_to_f32(&[i16::MIN, 0, i16::MAX]);
        assert!((v[0] - -1.0).abs() < f32::EPSILON);
        assert_eq!(v[1], 0.0);
        // i16::MAX maps to ~0.9999695 (32767/32768).
        assert!(v[2] < 1.0 && v[2] > 0.999);
    }

    #[test]
    fn duration_matches_sample_count() {
        let samples = vec![0_i16; SAMPLE_RATE_HZ as usize * 3];
        assert!((duration_seconds(&samples) - 3.0).abs() < f32::EPSILON);
    }
}
