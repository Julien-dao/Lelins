//! Text-to-speech abstraction for ANDREA.
//!
//! Real synthesis is delegated to a **Piper** binary running as a Tauri
//! sidecar. The desktop layer pipes ANDREA's streamed text into Piper one
//! sentence at a time and gets back raw PCM, which it queues into the
//! WebView audio buffer.
//!
//! This crate exposes:
//!
//! - the [`Synthesizer`] trait that the orchestration code consumes,
//! - a [`PiperSynthesizer`] that drives the official binary,
//! - a [`MockSynthesizer`] for tests.
//!
//! # PCM convention
//!
//! Piper's `--output_raw` flag emits **22 050 Hz mono 16-bit signed LE**.
//! The sample rate is voice-dependent (some voices are 16 kHz); each
//! [`AudioBuffer`] reports its own rate.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod mock;
mod piper;
mod types;

pub use mock::MockSynthesizer;
pub use piper::{PiperConfig, PiperSynthesizer};
pub use types::{AudioBuffer, SynthesizeError};

use async_trait::async_trait;

/// Text-to-speech engine abstraction.
#[async_trait]
pub trait Synthesizer: Send + Sync {
    /// Synthesize `text` into a single [`AudioBuffer`].
    ///
    /// Implementations should be pure (no global state) so multiple calls
    /// can be in flight concurrently.
    async fn synthesize(&self, text: &str) -> Result<AudioBuffer, SynthesizeError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_implements_trait() {
        let s: Box<dyn Synthesizer> = Box::new(MockSynthesizer::new(22_050));
        let buf = s.synthesize("Bonjour").await.unwrap();
        assert_eq!(buf.sample_rate, 22_050);
    }
}
