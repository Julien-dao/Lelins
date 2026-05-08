//! Conversation engine: glues LLM, STT, and TTS together.

use std::sync::Mutex;

use futures_util::StreamExt;
use thiserror::Error;

use andrea_llm::{ChatMessage, GenerateOptions, GenerateRequest, LlmError, LlmProvider};
use andrea_stt::{TranscribeError, TranscribeOptions, Transcriber};
use andrea_tts::{AudioBuffer, SynthesizeError, Synthesizer};

use crate::sentence::split_into_sentences;

/// Configuration for [`ConversationEngine`].
#[derive(Debug, Clone)]
pub struct ConversationConfig {
    /// Ollama model identifier (e.g. `mistral-small3.2:24b`).
    pub model: String,
    /// System prompt rendered once and prepended to every turn.
    pub system_prompt: String,
    /// LLM sampling options.
    pub options: GenerateOptions,
    /// Maximum number of `(user, assistant)` pairs kept in history.
    /// Older pairs are dropped from the prompt to bound context size.
    pub max_history_turns: usize,
    /// STT options (language, beam size, …).
    pub stt_options: TranscribeOptions,
}

impl ConversationConfig {
    /// Reasonable defaults for the v1 ANDREA Découverte tier.
    pub fn andrea_default(model: impl Into<String>, system_prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            system_prompt: system_prompt.into(),
            options: GenerateOptions::default(),
            max_history_turns: 8,
            stt_options: TranscribeOptions::default(),
        }
    }
}

/// Result of a single text-mode turn.
#[derive(Debug, Clone)]
pub struct TextTurn {
    /// Full assistant response, concatenated from all streamed tokens.
    pub answer: String,
    /// Total tokens emitted by the model (informational, may be 0).
    pub _token_count: usize,
}

/// Result of a single voice-mode turn.
#[derive(Debug)]
pub struct VoiceTurn {
    /// What the apprenant said, as decoded by the STT.
    pub transcript: String,
    /// ANDREA's full text response.
    pub answer: String,
    /// One audio buffer per synthesized sentence.
    pub audio: Vec<AudioBuffer>,
}

/// Errors raised by the orchestration layer.
#[derive(Debug, Error)]
pub enum ConversationError {
    /// LLM-side failure.
    #[error(transparent)]
    Llm(#[from] LlmError),
    /// STT-side failure.
    #[error(transparent)]
    Stt(#[from] TranscribeError),
    /// TTS-side failure.
    #[error(transparent)]
    Tts(#[from] SynthesizeError),
}

/// The engine that orchestrates one apprenant ↔ ANDREA exchange.
pub struct ConversationEngine<L, T, S>
where
    L: LlmProvider,
    T: Transcriber,
    S: Synthesizer,
{
    llm: L,
    stt: T,
    tts: S,
    config: ConversationConfig,
    history: Mutex<Vec<ChatMessage>>,
}

impl<L, T, S> ConversationEngine<L, T, S>
where
    L: LlmProvider,
    T: Transcriber,
    S: Synthesizer,
{
    /// Build an engine with explicit dependencies.
    pub fn new(llm: L, stt: T, tts: S, config: ConversationConfig) -> Self {
        Self {
            llm,
            stt,
            tts,
            config,
            history: Mutex::new(Vec::new()),
        }
    }

    /// Snapshot of the current conversation history.
    pub fn history(&self) -> Vec<ChatMessage> {
        self.history.lock().unwrap().clone()
    }

    /// Reset the conversation back to an empty history (system prompt
    /// is kept since it is rebuilt on every turn).
    pub fn reset(&self) {
        self.history.lock().unwrap().clear();
    }

    /// Run a single text-mode turn. Returns when the model emits its
    /// `done` frame; both the user message and the assistant reply are
    /// committed to history.
    pub async fn handle_text_turn(
        &self,
        user_message: impl Into<String>,
    ) -> Result<TextTurn, ConversationError> {
        let user = user_message.into();
        let request = self.build_request(&user);

        // Append the user message *now* so build_request() returns a clean
        // structure, but we still have it in history if the call fails halfway.
        let mut answer = String::new();
        let mut count = 0;
        let mut stream = self.llm.generate_stream(request).await?;
        while let Some(token) = stream.next().await {
            let token = token?;
            answer.push_str(&token.content);
            if !token.content.is_empty() {
                count += 1;
            }
            if token.done {
                break;
            }
        }

        // Commit history once the answer is materialized.
        let mut hist = self.history.lock().unwrap();
        hist.push(ChatMessage::user(user));
        hist.push(ChatMessage::assistant(answer.clone()));
        trim_history(&mut hist, self.config.max_history_turns);
        drop(hist);

        Ok(TextTurn {
            answer,
            _token_count: count,
        })
    }

    /// Run a single voice-mode turn:
    ///
    /// 1. transcribe the audio buffer (`samples` are 16 kHz mono i16),
    /// 2. run a text turn on the transcript,
    /// 3. synthesize the answer one sentence at a time so the UI can start
    ///    playback before the full response is ready.
    pub async fn handle_voice_turn(&self, samples: &[i16]) -> Result<VoiceTurn, ConversationError> {
        let transcript = self
            .stt
            .transcribe(samples, &self.config.stt_options)
            .await?;
        let user_text = transcript.text.trim().to_string();
        let text_turn = self.handle_text_turn(user_text.clone()).await?;
        let mut audio = Vec::new();
        for sentence in split_into_sentences(&text_turn.answer) {
            let buf = self.tts.synthesize(&sentence).await?;
            audio.push(buf);
        }
        Ok(VoiceTurn {
            transcript: user_text,
            answer: text_turn.answer,
            audio,
        })
    }

    fn build_request(&self, user_message: &str) -> GenerateRequest {
        let history = self.history.lock().unwrap();
        let mut messages = Vec::with_capacity(history.len() + 2);
        messages.push(ChatMessage::system(self.config.system_prompt.clone()));
        messages.extend_from_slice(&history);
        messages.push(ChatMessage::user(user_message.to_string()));
        GenerateRequest::new(self.config.model.clone(), messages)
            .with_options(self.config.options.clone())
    }
}

fn trim_history(history: &mut Vec<ChatMessage>, max_turns: usize) {
    let max_messages = max_turns * 2;
    if history.len() > max_messages {
        let to_remove = history.len() - max_messages;
        history.drain(0..to_remove);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use andrea_llm::{ChatRole, LlmError, Token};
    use andrea_stt::MockTranscriber;
    use andrea_tts::MockSynthesizer;
    use async_trait::async_trait;
    use futures_util::stream::{self, BoxStream};

    /// Minimal LLM mock: returns a fixed string as a single token, records
    /// the request it was called with so tests can assert on the prompt.
    struct MockLlm {
        reply: String,
        last_request: Mutex<Option<GenerateRequest>>,
    }

    impl MockLlm {
        fn new(reply: impl Into<String>) -> Self {
            Self {
                reply: reply.into(),
                last_request: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlm {
        async fn generate_stream(
            &self,
            request: GenerateRequest,
        ) -> Result<BoxStream<'static, Result<Token, LlmError>>, LlmError> {
            *self.last_request.lock().unwrap() = Some(request);
            let chunks = vec![
                Ok(Token {
                    content: self.reply.clone(),
                    done: false,
                    total_duration_ns: None,
                }),
                Ok(Token {
                    content: String::new(),
                    done: true,
                    total_duration_ns: Some(1234),
                }),
            ];
            Ok(stream::iter(chunks).boxed())
        }
        async fn embed(&self, _model: &str, _text: &str) -> Result<Vec<f32>, LlmError> {
            Ok(vec![0.0])
        }
        async fn health(&self) -> Result<(), LlmError> {
            Ok(())
        }
    }

    fn make_engine(reply: &str) -> ConversationEngine<MockLlm, MockTranscriber, MockSynthesizer> {
        let config = ConversationConfig::andrea_default(
            "mistral-small3.2:24b",
            "Tu es ANDREA, formatrice virtuelle.",
        );
        ConversationEngine::new(
            MockLlm::new(reply),
            MockTranscriber::new("Bonjour ANDREA"),
            MockSynthesizer::new(22_050),
            config,
        )
    }

    #[tokio::test]
    async fn text_turn_collects_response_and_records_history() {
        let engine = make_engine("Bonjour Marie ! Comment allez-vous ?");
        let turn = engine.handle_text_turn("Bonjour ANDREA").await.unwrap();
        assert_eq!(turn.answer, "Bonjour Marie ! Comment allez-vous ?");
        let hist = engine.history();
        assert_eq!(hist.len(), 2);
        assert_eq!(hist[0].role, ChatRole::User);
        assert_eq!(hist[0].content, "Bonjour ANDREA");
        assert_eq!(hist[1].role, ChatRole::Assistant);
    }

    #[tokio::test]
    async fn build_request_prepends_system_prompt_and_history() {
        let engine = make_engine("Salut.");
        engine.handle_text_turn("Premier message").await.unwrap();
        let _ = engine.handle_text_turn("Deuxième message").await;
        let last = engine
            .llm
            .last_request
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone();
        assert_eq!(last.messages[0].role, ChatRole::System);
        // history (2 prior messages + 1 user) = 3 + system = 4
        assert_eq!(last.messages.len(), 4);
        assert_eq!(last.messages[1].content, "Premier message");
        assert_eq!(last.messages[2].content, "Salut.");
        assert_eq!(last.messages[3].content, "Deuxième message");
    }

    #[tokio::test]
    async fn history_is_trimmed_to_max_turns() {
        let mut engine = make_engine("ok.");
        engine.config.max_history_turns = 2;
        for i in 0..5 {
            engine
                .handle_text_turn(format!("message {i}"))
                .await
                .unwrap();
        }
        let hist = engine.history();
        // 2 turns × 2 messages = 4
        assert_eq!(hist.len(), 4);
        // The oldest preserved user message is "message 3".
        assert_eq!(hist[0].content, "message 3");
    }

    #[tokio::test]
    async fn reset_clears_history_but_keeps_config() {
        let engine = make_engine("ok.");
        engine.handle_text_turn("hi").await.unwrap();
        assert!(!engine.history().is_empty());
        engine.reset();
        assert!(engine.history().is_empty());
    }

    #[tokio::test]
    async fn voice_turn_runs_full_pipeline() {
        let engine = make_engine("Première phrase. Deuxième phrase ! Troisième phrase ?");
        let samples = vec![100_i16; 32_000]; // 2 seconds of fake audio
        let turn = engine.handle_voice_turn(&samples).await.unwrap();
        assert_eq!(turn.transcript, "Bonjour ANDREA");
        assert_eq!(
            turn.answer,
            "Première phrase. Deuxième phrase ! Troisième phrase ?"
        );
        assert_eq!(turn.audio.len(), 3, "got {} buffers", turn.audio.len());
        // Mock synthesizer recorded each sentence text.
        assert_eq!(engine.tts.calls().len(), 3);
        // Mock transcriber received the full buffer.
        assert_eq!(engine.stt.calls()[0].samples_len, 32_000);
    }

    #[tokio::test]
    async fn options_default_match_andrea_doc() {
        let cfg = ConversationConfig::andrea_default("m", "p");
        assert!((cfg.options.temperature - 0.3).abs() < f32::EPSILON);
        assert_eq!(cfg.options.num_ctx, Some(8192));
    }
}
