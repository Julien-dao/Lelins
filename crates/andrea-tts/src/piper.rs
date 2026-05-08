//! Piper sidecar driver.
//!
//! Spawns a Piper process per call, sends the text on stdin, and reads raw
//! PCM from stdout. This is intentionally simple — pipe-per-utterance —
//! because each spoken sentence is short and the spawn cost is well
//! amortized by the Piper inference itself. A future optimization can
//! switch to `--json-input` for a long-running daemon.
//!
//! The configuration is split out into [`PiperConfig`] so tests can drop in
//! `cat` as a stand-in binary without depending on a real Piper install.

use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

use crate::types::{pcm_le_bytes_to_i16, AudioBuffer, SynthesizeError};
use crate::Synthesizer;

/// Configuration for [`PiperSynthesizer`].
#[derive(Debug, Clone)]
pub struct PiperConfig {
    /// Absolute path to the `piper` binary (or any compatible stand-in).
    pub binary: PathBuf,
    /// Arguments passed to the binary. Defaults to
    /// `["--model", "<voice>", "--output_raw"]` when built via [`Self::piper`].
    pub args: Vec<String>,
    /// Sample rate the binary is configured to emit, in Hz.
    pub sample_rate: u32,
    /// Number of channels (Piper voices are mono).
    pub channels: u16,
}

impl PiperConfig {
    /// Convenience for the default Piper invocation.
    ///
    /// `voice_path` is the `.onnx` voice file (Piper additionally reads the
    /// adjacent `.json` config automatically).
    pub fn piper(binary: impl Into<PathBuf>, voice_path: impl Into<PathBuf>) -> Self {
        let voice = voice_path.into();
        Self {
            binary: binary.into(),
            args: vec![
                "--model".to_string(),
                voice.display().to_string(),
                "--output_raw".to_string(),
            ],
            sample_rate: 22_050,
            channels: 1,
        }
    }
}

/// Synthesizer that pipes text through a Piper-compatible binary.
#[derive(Debug, Clone)]
pub struct PiperSynthesizer {
    config: PiperConfig,
}

impl PiperSynthesizer {
    /// Build a synthesizer from a configuration.
    pub fn new(config: PiperConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Synthesizer for PiperSynthesizer {
    async fn synthesize(&self, text: &str) -> Result<AudioBuffer, SynthesizeError> {
        if text.trim().is_empty() {
            return Err(SynthesizeError::EmptyText);
        }
        let mut child = Command::new(&self.config.binary)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| SynthesizeError::Spawn(e.to_string()))?;

        // Write the text and close stdin so the binary knows we're done.
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).await?;
            stdin.shutdown().await?;
        }

        // Read stdout and stderr concurrently.
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdout_task = async {
            if let Some(mut s) = stdout {
                s.read_to_end(&mut stdout_buf).await?;
            }
            Ok::<Vec<u8>, std::io::Error>(stdout_buf)
        };
        let stderr_task = async {
            if let Some(mut s) = stderr {
                s.read_to_end(&mut stderr_buf).await?;
            }
            Ok::<Vec<u8>, std::io::Error>(stderr_buf)
        };

        let (stdout_bytes, stderr_bytes, status) =
            tokio::try_join!(stdout_task, stderr_task, async { child.wait().await })?;

        if !status.success() {
            return Err(SynthesizeError::ProcessExit {
                status: status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&stderr_bytes).into_owned(),
            });
        }

        let samples = pcm_le_bytes_to_i16(&stdout_bytes)?;
        Ok(AudioBuffer {
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
            samples,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Use `cat` as a stand-in for `piper`: the bytes we send on stdin come
    /// back unchanged on stdout, which lets us validate the pipe wiring and
    /// the PCM parser end-to-end without depending on Piper.
    #[tokio::test]
    #[cfg(unix)]
    async fn cat_roundtrip_returns_input_as_pcm() {
        let config = PiperConfig {
            binary: "cat".into(),
            args: vec![],
            sample_rate: 22_050,
            channels: 1,
        };
        let synth = PiperSynthesizer::new(config);
        // 4 bytes -> 2 i16 samples (avoids the malformed-PCM error).
        let buf = synth.synthesize("ABCD").await.unwrap();
        assert_eq!(buf.samples.len(), 2);
        // bytes [0x41, 0x42] LE = 0x4241, [0x43, 0x44] LE = 0x4443
        assert_eq!(buf.samples[0], 0x4241);
        assert_eq!(buf.samples[1], 0x4443);
        assert_eq!(buf.sample_rate, 22_050);
        assert_eq!(buf.channels, 1);
    }

    #[tokio::test]
    async fn missing_binary_returns_spawn_error() {
        let config = PiperConfig {
            binary: "/no/such/binary-for-andrea-tests".into(),
            args: vec![],
            sample_rate: 22_050,
            channels: 1,
        };
        let synth = PiperSynthesizer::new(config);
        let err = synth.synthesize("hi").await.unwrap_err();
        assert!(matches!(err, SynthesizeError::Spawn(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn empty_text_short_circuits() {
        let config = PiperConfig {
            binary: "cat".into(),
            args: vec![],
            sample_rate: 22_050,
            channels: 1,
        };
        let synth = PiperSynthesizer::new(config);
        assert!(matches!(
            synth.synthesize("").await,
            Err(SynthesizeError::EmptyText)
        ));
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn nonzero_exit_is_surfaced() {
        let config = PiperConfig {
            binary: "false".into(),
            args: vec![],
            sample_rate: 22_050,
            channels: 1,
        };
        let synth = PiperSynthesizer::new(config);
        let err = synth.synthesize("hi").await.unwrap_err();
        assert!(
            matches!(err, SynthesizeError::ProcessExit { .. }),
            "got {err:?}"
        );
    }

    #[test]
    fn piper_config_default_args_use_voice_path() {
        let cfg = PiperConfig::piper("/usr/bin/piper", "/data/voices/fr_FR-siwis-medium.onnx");
        assert_eq!(cfg.args[0], "--model");
        assert!(cfg.args[1].ends_with("fr_FR-siwis-medium.onnx"));
        assert_eq!(cfg.args[2], "--output_raw");
        assert_eq!(cfg.sample_rate, 22_050);
        assert_eq!(cfg.channels, 1);
    }
}
