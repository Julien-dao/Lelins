//! Conversation orchestration for ANDREA.
//!
//! Wires the LLM (`andrea-llm`), STT (`andrea-stt`), and TTS (`andrea-tts`)
//! engines into a single [`ConversationEngine`]. The engine owns:
//!
//! - the system prompt (the "ANDREA Formateur v1" template — eventually
//!   loaded from `packages/prompts/`),
//! - the chat history with a sliding-window cap to keep the prompt below
//!   the model's `num_ctx`,
//! - configuration (model name, max history, sentence-splitter behaviour).
//!
//! It exposes high-level entry points used by the Tauri commands:
//!
//! - [`ConversationEngine::handle_text_turn`] — pure text round-trip.
//! - [`ConversationEngine::handle_voice_turn`] — STT → LLM → TTS by sentence.
//!
//! RAG retrieval and the full Knowles-aware prompt are added in steps 3–4.
//! The orchestration here is engine-agnostic and tested against mocks.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod engine;
mod sentence;

pub use engine::{ConversationConfig, ConversationEngine, ConversationError, TextTurn, VoiceTurn};
pub use sentence::split_into_sentences;

// Re-export the trait types so callers do not need to import all four crates.
pub use andrea_llm::{ChatMessage, ChatRole, GenerateOptions, LlmProvider};
pub use andrea_stt::{TranscribeOptions, Transcriber};
pub use andrea_tts::{AudioBuffer, Synthesizer};
