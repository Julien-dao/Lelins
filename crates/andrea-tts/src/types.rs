//! Shared types for the TTS abstraction.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A chunk of synthesized audio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioBuffer {
    /// Sample rate in hertz (depends on the voice).
    pub sample_rate: u32,
    /// Number of channels (Piper voices are mono).
    pub channels: u16,
    /// PCM samples, mono interleaved if channels > 1.
    pub samples: Vec<i16>,
}

impl AudioBuffer {
    /// Duration of the buffer in seconds.
    pub fn duration_seconds(&self) -> f32 {
        if self.sample_rate == 0 || self.channels == 0 {
            return 0.0;
        }
        let frames = self.samples.len() as f32 / self.channels as f32;
        frames / self.sample_rate as f32
    }

    /// Fill the buffer with silence of the given duration. Useful for tests
    /// and as a fallback when synthesis fails.
    pub fn silence(sample_rate: u32, channels: u16, duration_seconds: f32) -> Self {
        let frames = (sample_rate as f32 * duration_seconds).round() as usize;
        Self {
            sample_rate,
            channels,
            samples: vec![0_i16; frames * channels as usize],
        }
    }
}

/// Errors raised by [`crate::Synthesizer`] implementations.
#[derive(Debug, Error)]
pub enum SynthesizeError {
    /// Empty input text.
    #[error("text to synthesize was empty")]
    EmptyText,
    /// Spawning the sidecar process failed (binary missing, permissions).
    #[error("failed to spawn TTS process: {0}")]
    Spawn(String),
    /// The sidecar reported a non-zero exit status.
    #[error("TTS process exited with status {status}: {stderr}")]
    ProcessExit {
        /// Reported exit code (or `-1` if unavailable).
        status: i32,
        /// Trailing standard-error output for diagnostics.
        stderr: String,
    },
    /// I/O error talking to the sidecar.
    #[error("TTS I/O error: {0}")]
    Io(String),
    /// PCM output had an odd byte count or otherwise malformed framing.
    #[error("malformed PCM output: {0}")]
    MalformedPcm(String),
}

impl From<std::io::Error> for SynthesizeError {
    fn from(e: std::io::Error) -> Self {
        SynthesizeError::Io(e.to_string())
    }
}

/// Convert a contiguous little-endian `i16` byte stream into `Vec<i16>`.
///
/// Returns [`SynthesizeError::MalformedPcm`] if the byte count is odd.
pub fn pcm_le_bytes_to_i16(bytes: &[u8]) -> Result<Vec<i16>, SynthesizeError> {
    if bytes.len() % 2 != 0 {
        return Err(SynthesizeError::MalformedPcm(format!(
            "odd byte count: {}",
            bytes.len()
        )));
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        out.push(i16::from_le_bytes([chunk[0], chunk[1]]));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_for_silence_matches_request() {
        let buf = AudioBuffer::silence(22_050, 1, 1.0);
        assert!((buf.duration_seconds() - 1.0).abs() < 1e-3);
        assert_eq!(buf.samples.len(), 22_050);
    }

    #[test]
    fn pcm_le_round_trip() {
        let original: Vec<i16> = vec![-1000, 0, 1000, i16::MAX, i16::MIN];
        let bytes: Vec<u8> = original.iter().flat_map(|s| s.to_le_bytes()).collect();
        let parsed = pcm_le_bytes_to_i16(&bytes).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn pcm_le_rejects_odd_length() {
        let err = pcm_le_bytes_to_i16(&[1, 2, 3]).unwrap_err();
        assert!(matches!(err, SynthesizeError::MalformedPcm(_)));
    }
}
