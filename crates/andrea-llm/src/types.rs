//! Shared types for the LLM provider abstraction.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Role a chat message plays in the conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    /// System / instruction prompt.
    System,
    /// Message authored by the human apprenant.
    User,
    /// Message authored by ANDREA.
    Assistant,
}

/// Single message in a chat-style prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Author role.
    pub role: ChatRole,
    /// UTF-8 text content.
    pub content: String,
}

impl ChatMessage {
    /// Build a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::System,
            content: content.into(),
        }
    }
    /// Build a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::User,
            content: content.into(),
        }
    }
    /// Build an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content.into(),
        }
    }
}

/// Sampling and decoding parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOptions {
    /// Sampling temperature. ANDREA defaults to 0.3 for pedagogical precision.
    pub temperature: f32,
    /// Nucleus sampling parameter.
    pub top_p: f32,
    /// Penalty applied to repeated tokens.
    pub repeat_penalty: f32,
    /// Maximum new tokens to predict (`None` = let the model decide / hit context).
    pub num_predict: Option<u32>,
    /// Context window size (in tokens). `None` = model default.
    pub num_ctx: Option<u32>,
    /// Optional stop sequences.
    pub stop: Vec<String>,
}

impl Default for GenerateOptions {
    /// ANDREA defaults: low temperature, sensible cap on response length.
    fn default() -> Self {
        Self {
            temperature: 0.3,
            top_p: 0.9,
            repeat_penalty: 1.15,
            num_predict: Some(800),
            num_ctx: Some(8192),
            stop: vec![],
        }
    }
}

/// A chat-style request.
#[derive(Debug, Clone)]
pub struct GenerateRequest {
    /// Model name as known to Ollama (`mistral-small3.2:24b` etc.).
    pub model: String,
    /// Conversation messages, oldest first.
    pub messages: Vec<ChatMessage>,
    /// Sampling options.
    pub options: GenerateOptions,
}

impl GenerateRequest {
    /// Build a request with default options for the given model and messages.
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            options: GenerateOptions::default(),
        }
    }

    /// Replace sampling options.
    pub fn with_options(mut self, options: GenerateOptions) -> Self {
        self.options = options;
        self
    }
}

/// A streamed token emitted by the provider.
#[derive(Debug, Clone)]
pub struct Token {
    /// Text fragment to append to the running response.
    pub content: String,
    /// `true` for the final chunk of the stream.
    pub done: bool,
    /// Total wall-clock duration so far in nanoseconds (when reported).
    pub total_duration_ns: Option<u64>,
}

/// Embedding response from `/api/embeddings`.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbedResponse {
    /// The embedding vector.
    pub embedding: Vec<f32>,
}

/// Provider-level errors.
#[derive(Debug, Error)]
pub enum LlmError {
    /// Provider not reachable (typically Ollama still starting or crashed).
    #[error("LLM provider unavailable: {0}")]
    Unavailable(String),
    /// Provider returned a non-2xx HTTP status.
    #[error("LLM provider error {status}: {body}")]
    Http {
        /// HTTP status code.
        status: u16,
        /// Body returned by the provider, truncated to a sane length.
        body: String,
    },
    /// JSON serialization failure (request build or response parse).
    #[error("LLM serialization error: {0}")]
    Serde(String),
    /// Internal consistency violation (e.g. unexpected stream framing).
    #[error("LLM protocol error: {0}")]
    Protocol(String),
    /// Underlying transport error.
    #[error("LLM transport error: {0}")]
    Transport(String),
}

impl From<reqwest::Error> for LlmError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_connect() || e.is_timeout() {
            LlmError::Unavailable(e.to_string())
        } else {
            LlmError::Transport(e.to_string())
        }
    }
}

impl From<serde_json::Error> for LlmError {
    fn from(e: serde_json::Error) -> Self {
        LlmError::Serde(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_matches_andrea_doc() {
        let o = GenerateOptions::default();
        assert!((o.temperature - 0.3).abs() < f32::EPSILON);
        assert_eq!(o.num_ctx, Some(8192));
        assert_eq!(o.num_predict, Some(800));
    }

    #[test]
    fn chat_message_helpers() {
        let s = ChatMessage::system("hi");
        assert_eq!(s.role, ChatRole::System);
        assert_eq!(s.content, "hi");
        let u = ChatMessage::user("ping");
        assert_eq!(u.role, ChatRole::User);
        let a = ChatMessage::assistant("pong");
        assert_eq!(a.role, ChatRole::Assistant);
    }

    #[test]
    fn role_serializes_lowercase() {
        let json = serde_json::to_string(&ChatRole::Assistant).unwrap();
        assert_eq!(json, "\"assistant\"");
    }
}
