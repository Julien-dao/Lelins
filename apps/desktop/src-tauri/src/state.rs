//! Shared runtime state for ANDREA Tauri commands.
//!
//! Held inside `tauri::State<AppState>`. The `engine` field is wrapped in
//! `Arc` so commands can be invoked concurrently — the engine's internal
//! locking serializes mutation of conversation history.

use std::sync::Arc;

use andrea_conversation::{ConversationConfig, ConversationEngine};
use andrea_llm::OllamaProvider;
use andrea_stt::MockTranscriber;
use andrea_tts::MockSynthesizer;

/// Concrete engine type used in the desktop app.
///
/// The STT and TTS slots are mocks today. They will be swapped for the
/// real Whisper.cpp and Piper drivers in a follow-up sub-step once we
/// build on macOS — the `ConversationEngine` API does not change.
pub type AndreaEngine = ConversationEngine<OllamaProvider, MockTranscriber, MockSynthesizer>;

/// Shared state injected into every Tauri command.
pub struct AppState {
    /// The conversation engine, ready to handle text/voice turns.
    pub engine: Arc<AndreaEngine>,
    /// Server secret used by `andrea-license` to verify keys offline.
    pub license_secret: Vec<u8>,
}

impl AppState {
    /// Build state with sensible v1 defaults.
    ///
    /// - LLM : Ollama on `http://localhost:11434`.
    /// - STT : mock that returns a placeholder transcript.
    /// - TTS : mock that returns silence.
    pub fn from_env() -> Self {
        let llm = OllamaProvider::local_default();
        let stt = MockTranscriber::new("[STT non configurée — Whisper sera branché en sous-étape suivante]");
        let tts = MockSynthesizer::new(22_050);
        let config = ConversationConfig::andrea_default(
            "mistral-small3.2:24b",
            DEFAULT_SYSTEM_PROMPT,
        );
        let engine = Arc::new(ConversationEngine::new(llm, stt, tts, config));

        let license_secret = option_env!("ANDREA_LICENSE_SECRET")
            .map(|s| s.as_bytes().to_vec())
            .unwrap_or_else(|| b"andrea-dev-placeholder-secret-do-not-ship".to_vec());

        Self {
            engine,
            license_secret,
        }
    }
}

/// Placeholder system prompt for sub-step 2.G.
///
/// The full ANDREA Formateur v1 prompt (see `docs/03-systeme-prompt-...md`)
/// is loaded from `packages/prompts/` in step 4 and rendered with profile
/// variables. For now this short version is enough to validate the
/// end-to-end pipeline.
const DEFAULT_SYSTEM_PROMPT: &str = "\
Tu es ANDREA, formatrice virtuelle experte en andragogie qui prépare les \
apprenants au Titre Professionnel Formateur Professionnel d'Adultes \
(RNCP n°37275, certifié par le Ministère du Travail). \
Tu es bienveillamment exigeante. \
Tu mobilises systématiquement les principes de Knowles. \
Tu n'inventes JAMAIS de référence du référentiel et tu cites toujours la source.";
