//! Shared runtime state for ANDREA Tauri commands.
//!
//! Held inside `tauri::State<AppState>`. The `engine` field is wrapped in
//! `Arc` so commands can be invoked concurrently — the engine's internal
//! locking serializes mutation of conversation history.

use std::sync::Arc;

use andrea_conversation::{ConversationConfig, ConversationEngine, RagRetriever};
use andrea_llm::OllamaProvider;
use andrea_rag::{ingest::ingest_documents, InMemoryVectorStore, KeywordEmbedder};
use andrea_referentiel::build_initial_chunks;
use andrea_stt::MockTranscriber;
use andrea_tts::MockSynthesizer;

/// Concrete engine type used in the desktop app.
///
/// The STT and TTS slots are mocks today. They will be swapped for the
/// real Whisper.cpp and Piper drivers in a follow-up sub-step once we
/// build on macOS — the `ConversationEngine` API does not change.
pub type AndreaEngine = ConversationEngine<OllamaProvider, MockTranscriber, MockSynthesizer>;

/// Embedding dimension used by the in-memory store. Matches `bge-m3` so
/// production embeddings drop in without resizing.
const EMBEDDING_DIM: usize = 1024;

/// Shared state injected into every Tauri command.
pub struct AppState {
    /// The conversation engine, ready to handle text/voice turns.
    pub engine: Arc<AndreaEngine>,
    /// Embedder used by RAG. Today: a deterministic [`KeywordEmbedder`]
    /// for offline dev. Once Ollama is running the desktop app will swap
    /// in a wrapper around `LlmProvider::embed` that targets `bge-m3`.
    pub embedder: Arc<KeywordEmbedder>,
    /// Vector store holding the FPA référentiel chunks.
    pub vector_store: Arc<InMemoryVectorStore>,
    /// Server secret used by `andrea-license` to verify keys offline.
    pub license_secret: Vec<u8>,
}

impl AppState {
    /// Build state with sensible v1 defaults.
    ///
    /// - LLM      : Ollama on `http://localhost:11434`.
    /// - STT      : mock that returns a placeholder transcript.
    /// - TTS      : mock that returns silence.
    /// - Embedder : KeywordEmbedder placeholder (swap for Ollama bge-m3
    ///   once the model is downloaded).
    pub fn from_env() -> Self {
        let llm = OllamaProvider::local_default();
        let stt = MockTranscriber::new(
            "[STT non configurée — Whisper sera branché en sous-étape suivante]",
        );
        let tts = MockSynthesizer::new(22_050);
        let embedder = Arc::new(KeywordEmbedder::new(EMBEDDING_DIM));
        let vector_store = Arc::new(InMemoryVectorStore::new(EMBEDDING_DIM));
        let retriever = RagRetriever::new(
            embedder.clone() as Arc<dyn andrea_rag::EmbeddingProvider>,
            vector_store.clone() as Arc<dyn andrea_rag::VectorStore>,
        );
        let config = ConversationConfig::andrea_default(
            "mistral-small3.2:24b",
            DEFAULT_SYSTEM_PROMPT,
        );
        let engine = Arc::new(ConversationEngine::with_retriever(
            llm,
            stt,
            tts,
            config,
            Box::new(retriever),
        ));

        let license_secret = option_env!("ANDREA_LICENSE_SECRET")
            .map(|s| s.as_bytes().to_vec())
            .unwrap_or_else(|| b"andrea-dev-placeholder-secret-do-not-ship".to_vec());

        Self {
            engine,
            embedder,
            vector_store,
            license_secret,
        }
    }

    /// Ingest the bootstrap FPA référentiel chunks into the vector store.
    ///
    /// Idempotent — safe to call multiple times. Designed to be invoked
    /// from the Tauri `setup` hook so the corpus is available before the
    /// user sends their first message.
    pub async fn ingest_bootstrap_corpus(&self) -> Result<(), String> {
        let chunks = build_initial_chunks();
        ingest_documents(&*self.embedder, &*self.vector_store, chunks, |_, _| {})
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Placeholder system prompt for sub-step 3.D.
///
/// The full ANDREA Formateur v1 prompt (see `docs/03-systeme-prompt-...md`)
/// is loaded from `packages/prompts/` in step 4 and rendered with profile
/// variables. For now this short version is enough to validate the
/// end-to-end pipeline including RAG augmentation.
const DEFAULT_SYSTEM_PROMPT: &str = "\
Tu es ANDREA, formatrice virtuelle experte en andragogie qui prépare les \
apprenants au Titre Professionnel Formateur Professionnel d'Adultes \
(RNCP n°37275, certifié par le Ministère du Travail). \
Tu es bienveillamment exigeante. \
Tu mobilises systématiquement les principes de Knowles. \
Tu n'inventes JAMAIS de référence du référentiel et tu cites toujours la source. \
Quand tu disposes d'extraits du référentiel sous le bloc « EXTRAITS DU \
RÉFÉRENTIEL », tu les cites explicitement dans ta réponse au format \
« Source : REAC V07 21/12/2022, CCPx, CPy ». \
Si aucun extrait pertinent n'est fourni, tu indiques que ta réponse \
s'appuie sur ta connaissance générale du métier et tu invites l'apprenant \
à vérifier sur la fiche officielle France Compétences.";
