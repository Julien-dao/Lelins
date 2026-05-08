//! Shared types for the STT abstraction.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Options controlling the transcription pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscribeOptions {
    /// IETF-BCP47 language tag (`fr`, `en`, …). `None` lets the engine auto-detect.
    pub language: Option<String>,
    /// If `true`, translate non-`en` content to English (Whisper's translate mode).
    /// ANDREA always wants `false` — kept for completeness.
    pub translate: bool,
    /// Number of beams in beam search (1 = greedy, 5 = Whisper default).
    pub beam_size: u32,
    /// Optional initial prompt to bias decoding (e.g. "Le titre FPA…").
    pub initial_prompt: Option<String>,
    /// If `true`, keep word-level timestamps. Costs CPU.
    pub word_timestamps: bool,
}

impl Default for TranscribeOptions {
    /// ANDREA defaults: French, no translation, greedy decoding, no timestamps.
    fn default() -> Self {
        Self {
            language: Some("fr".to_string()),
            translate: false,
            beam_size: 1,
            initial_prompt: None,
            word_timestamps: false,
        }
    }
}

/// One contiguous segment of the transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    /// Start offset in seconds from the beginning of the audio.
    pub start_s: f32,
    /// End offset in seconds.
    pub end_s: f32,
    /// Decoded text, trimmed.
    pub text: String,
}

/// Full transcript returned by [`crate::Transcriber::transcribe`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    /// Concatenated text of all segments, trimmed.
    pub text: String,
    /// Individual segments (always at least one if `text` is non-empty).
    pub segments: Vec<Segment>,
    /// Detected or supplied language code (e.g. `"fr"`).
    pub language: String,
    /// Total wall-clock duration of the inference in seconds.
    pub processing_seconds: f32,
}

/// Errors raised by [`crate::Transcriber`] implementations.
#[derive(Debug, Error)]
pub enum TranscribeError {
    /// The audio buffer was empty or below a minimum threshold.
    #[error("audio buffer too short ({samples} samples, need at least {min})")]
    BufferTooShort {
        /// Samples received.
        samples: usize,
        /// Minimum samples required.
        min: usize,
    },
    /// Underlying engine failed.
    #[error("STT engine error: {0}")]
    Engine(String),
    /// I/O error (e.g. loading the model file).
    #[error("STT I/O error: {0}")]
    Io(String),
    /// The configured engine is not compiled in this build.
    #[error("STT engine `{0}` is not available in this build")]
    UnavailableEngine(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_are_french_no_translate() {
        let o = TranscribeOptions::default();
        assert_eq!(o.language.as_deref(), Some("fr"));
        assert!(!o.translate);
        assert_eq!(o.beam_size, 1);
    }

    #[test]
    fn transcript_serializes() {
        let t = Transcript {
            text: "Bonjour".to_string(),
            segments: vec![Segment {
                start_s: 0.0,
                end_s: 0.5,
                text: "Bonjour".to_string(),
            }],
            language: "fr".to_string(),
            processing_seconds: 0.12,
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"text\":\"Bonjour\""));
    }
}
