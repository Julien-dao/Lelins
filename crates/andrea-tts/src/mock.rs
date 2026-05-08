//! In-memory synthesizer for tests.

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::types::{AudioBuffer, SynthesizeError};
use crate::Synthesizer;

/// Synthesizer that returns 0.5 s of silence and records every call.
#[derive(Debug, Clone)]
pub struct MockSynthesizer {
    sample_rate: u32,
    calls: Arc<Mutex<Vec<String>>>,
}

impl MockSynthesizer {
    /// New mock that produces silence at `sample_rate` Hz mono.
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Snapshot of every text passed to [`Synthesizer::synthesize`].
    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl Synthesizer for MockSynthesizer {
    async fn synthesize(&self, text: &str) -> Result<AudioBuffer, SynthesizeError> {
        if text.trim().is_empty() {
            return Err(SynthesizeError::EmptyText);
        }
        self.calls.lock().unwrap().push(text.to_string());
        Ok(AudioBuffer::silence(self.sample_rate, 1, 0.5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_returns_silence_and_records_text() {
        let mock = MockSynthesizer::new(22_050);
        let buf = mock.synthesize("bonjour").await.unwrap();
        assert_eq!(buf.sample_rate, 22_050);
        assert!(buf.duration_seconds() > 0.0);
        assert_eq!(mock.calls(), vec!["bonjour".to_string()]);
    }

    #[tokio::test]
    async fn mock_rejects_empty_text() {
        let mock = MockSynthesizer::new(22_050);
        assert!(matches!(
            mock.synthesize("   ").await,
            Err(SynthesizeError::EmptyText)
        ));
    }
}
