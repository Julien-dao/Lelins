//! LLM provider abstraction for ANDREA.
//!
//! The desktop app talks to a local Ollama process running as a sidecar.
//! All access is mediated through the [`LlmProvider`] trait so that the
//! storage backend can be swapped to `mistral.rs`, `llama.cpp` direct, or
//! any future engine without touching the orchestration layer.
//!
//! # Streaming
//!
//! Generation is exposed as an async stream of [`Token`]s. Each token
//! carries the cumulative `total_duration` (when known) and a `done` flag
//! that the consumer can use to flush the TTS pipeline.
//!
//! # Embeddings
//!
//! `embed` returns a single dense vector. The dimension depends on the
//! model: `bge-m3` returns 1024-dim vectors, `nomic-embed-text` 768.
//!
//! # Error model
//!
//! All errors funnel into [`LlmError`]. The HTTP client maps connection
//! refusals (Ollama not running) and timeouts to [`LlmError::Unavailable`]
//! so the UI can show a clear "ANDREA est en cours de démarrage…" state.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod ollama;
mod types;

pub use ollama::OllamaProvider;
pub use types::{
    ChatMessage, ChatRole, EmbedResponse, GenerateOptions, GenerateRequest, LlmError, Token,
};

use async_trait::async_trait;
use futures_util::stream::BoxStream;

/// LLM provider abstraction. All ANDREA orchestration depends on this trait.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Stream tokens from a prompt completion. The returned stream ends
    /// when the model emits `done = true` or the connection drops.
    async fn generate_stream(
        &self,
        request: GenerateRequest,
    ) -> Result<BoxStream<'static, Result<Token, LlmError>>, LlmError>;

    /// One-shot completion (collects the full text, no streaming).
    /// Convenience for short utility calls (session summarization, etc.).
    async fn generate(&self, request: GenerateRequest) -> Result<String, LlmError> {
        use futures_util::StreamExt;
        let mut stream = self.generate_stream(request).await?;
        let mut out = String::new();
        while let Some(token) = stream.next().await {
            let token = token?;
            out.push_str(&token.content);
            if token.done {
                break;
            }
        }
        Ok(out)
    }

    /// Embed `text` with the configured embedding model.
    async fn embed(&self, model: &str, text: &str) -> Result<Vec<f32>, LlmError>;

    /// Health-check (HEAD or `/api/version` ping).
    async fn health(&self) -> Result<(), LlmError>;
}
