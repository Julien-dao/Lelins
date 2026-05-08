//! In-memory [`crate::Transcriber`] used by tests and orchestration code
//! while the real Whisper.cpp implementation lives behind a feature flag.

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::types::{Segment, TranscribeError, TranscribeOptions, Transcript};
use crate::Transcriber;

/// Mock transcriber with a fixed reply for every call.
///
/// Records the inputs it was called with so tests can assert.
#[derive(Debug, Clone)]
pub struct MockTranscriber {
    fixed_reply: String,
    /// `Arc<Mutex<…>>` lets multiple owners share the same call log.
    calls: Arc<Mutex<Vec<MockCall>>>,
}

/// A single recorded call to [`MockTranscriber::transcribe`].
#[derive(Debug, Clone)]
pub struct MockCall {
    /// Number of samples passed in.
    pub samples_len: usize,
    /// Options used for the call.
    pub options: TranscribeOptions,
}

impl MockTranscriber {
    /// New mock that always returns `fixed_reply`.
    pub fn new(fixed_reply: impl Into<String>) -> Self {
        Self {
            fixed_reply: fixed_reply.into(),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Snapshot of all calls received so far.
    pub fn calls(&self) -> Vec<MockCall> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl Transcriber for MockTranscriber {
    async fn transcribe(
        &self,
        samples: &[i16],
        options: &TranscribeOptions,
    ) -> Result<Transcript, TranscribeError> {
        if samples.is_empty() {
            return Err(TranscribeError::BufferTooShort { samples: 0, min: 1 });
        }
        self.calls.lock().unwrap().push(MockCall {
            samples_len: samples.len(),
            options: options.clone(),
        });
        let duration = crate::duration_seconds(samples);
        Ok(Transcript {
            text: self.fixed_reply.clone(),
            segments: vec![Segment {
                start_s: 0.0,
                end_s: duration,
                text: self.fixed_reply.clone(),
            }],
            language: options.language.clone().unwrap_or_else(|| "fr".to_string()),
            processing_seconds: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_returns_fixed_reply_and_records_call() {
        let mock = MockTranscriber::new("Bonjour ANDREA");
        let samples = vec![0_i16; 16_000]; // 1 second of silence
        let opts = TranscribeOptions::default();
        let t = mock.transcribe(&samples, &opts).await.unwrap();
        assert_eq!(t.text, "Bonjour ANDREA");
        assert_eq!(t.language, "fr");
        assert_eq!(t.segments.len(), 1);
        assert!((t.segments[0].end_s - 1.0).abs() < f32::EPSILON);
        assert_eq!(mock.calls().len(), 1);
        assert_eq!(mock.calls()[0].samples_len, 16_000);
    }

    #[tokio::test]
    async fn mock_rejects_empty_buffer() {
        let mock = MockTranscriber::new("anything");
        let err = mock
            .transcribe(&[], &TranscribeOptions::default())
            .await
            .unwrap_err();
        assert!(matches!(err, TranscribeError::BufferTooShort { .. }));
    }

    #[tokio::test]
    async fn mock_can_be_shared_across_threads() {
        let mock = MockTranscriber::new("hi");
        let m2 = mock.clone();
        let handle = tokio::spawn(async move {
            let samples = vec![1_i16; 100];
            m2.transcribe(&samples, &TranscribeOptions::default())
                .await
                .unwrap();
        });
        handle.await.unwrap();
        assert_eq!(mock.calls().len(), 1);
    }
}
