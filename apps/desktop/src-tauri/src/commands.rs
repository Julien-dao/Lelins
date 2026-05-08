//! Tauri commands invoked by the React frontend.
//!
//! Each command is `async` to avoid blocking the runtime and returns a
//! serializable result. Domain errors are converted to strings here for
//! easy consumption from JS.

use andrea_hardware::{HardwareProfile, ModelChoice, MIN_SUPPORTED_RAM_GB};
use andrea_license::{verify, LicenseError, Tier};
use andrea_rag::{retrieve::search, VectorStore};
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

/// Trivial health-check used by the frontend at startup.
#[tauri::command]
pub async fn ping() -> &'static str {
    "pong"
}

// ---------- Licence ----------

/// Result returned by [`license_validate`].
#[derive(Debug, Clone, Serialize)]
pub struct LicenseSummary {
    /// Canonical key string (`ANDREA-...`).
    pub canonical: String,
    /// Tier label (`DECO` / `PRO` / `MAIT` / `BNDL`).
    pub tier: &'static str,
    /// Format version embedded in the payload.
    pub version: u8,
    /// Days since 2026-01-01 at issuance time.
    pub issued_days: u16,
    /// Feature bitmask.
    pub features: u16,
}

impl From<andrea_license::License> for LicenseSummary {
    fn from(l: andrea_license::License) -> Self {
        Self {
            canonical: l.canonical,
            tier: l.payload.tier.label(),
            version: l.payload.version,
            issued_days: l.payload.issued_days,
            features: l.payload.features,
        }
    }
}

/// Validate a user-typed key against the embedded server secret and the
/// installed user's email.
#[tauri::command]
pub async fn license_validate(
    state: State<'_, AppState>,
    raw_key: String,
    email: String,
) -> Result<LicenseSummary, String> {
    let revocation: &[&str] = &[];
    verify(&raw_key, &email, &state.license_secret, revocation)
        .map(LicenseSummary::from)
        .map_err(license_error_message)
}

/// Convenience: parse a key without verifying its MAC, useful for showing
/// the tier badge in the UI before the user has confirmed their email.
#[tauri::command]
pub async fn license_info(raw_key: String) -> Result<&'static str, String> {
    let normalized = andrea_license::normalize(&raw_key);
    if !normalized.starts_with("ANDREA") {
        return Err("La clé doit commencer par ANDREA.".to_string());
    }
    let after_brand = &normalized[6..];
    for tier_len in [4usize, 3] {
        if after_brand.len() >= tier_len {
            let label = &after_brand[..tier_len];
            if let Ok(tier) = Tier::from_label(label) {
                return Ok(tier.label());
            }
        }
    }
    Err("Tier inconnu.".to_string())
}

fn license_error_message(err: LicenseError) -> String {
    use LicenseError::*;
    match err {
        Format(_) => "Format de clé invalide. Vérifiez la saisie.".to_string(),
        Crockford(_) => "Caractère invalide dans la clé.".to_string(),
        Tier(_) => "Tier de licence non reconnu.".to_string(),
        TierMismatch { .. } => "Le tier indiqué ne correspond pas à la clé.".to_string(),
        UnsupportedVersion(v) => {
            format!("Cette version de clé ({v}) n'est pas reconnue. Mettez à jour ANDREA.")
        }
        BadMac => "La signature de la clé est invalide.".to_string(),
        EmailMismatch => "Cette clé ne correspond pas à l'email saisi.".to_string(),
        Revoked => "Cette clé a été révoquée. Contactez le support.".to_string(),
    }
}

// ---------- Hardware & model selection ----------

/// Return the detected hardware profile.
#[tauri::command]
pub async fn hardware_profile() -> HardwareProfile {
    andrea_hardware::detect()
}

/// Return the model recommended for the current hardware. On insufficient
/// RAM, returns a localised error message instead of a `ModelChoice`.
#[tauri::command]
pub async fn recommended_model() -> Result<ModelChoice, String> {
    andrea_hardware::auto_select().map_err(|e| match e {
        andrea_hardware::SelectionError::InsufficientRam { total_ram_gb, min } => {
            format!(
                "RAM insuffisante : {total_ram_gb} Go détectés, {min} Go requis. \
                 ANDREA ne peut pas s'installer sur cette machine."
            )
        }
    })
}

/// Static minimum supported RAM, in gibibytes. Useful for the UI to show
/// requirements without an extra round-trip.
#[tauri::command]
pub async fn minimum_ram_gb() -> u32 {
    MIN_SUPPORTED_RAM_GB
}

// ---------- Chat ----------

/// Result of a single text-mode chat turn.
#[derive(Debug, Clone, Serialize)]
pub struct ChatReply {
    /// Full assistant response.
    pub answer: String,
}

/// Send a text message to ANDREA. Blocks until the LLM emits its `done`
/// frame. Streaming via Tauri events will be added in a later sub-step
/// once the UI implements an event listener.
#[tauri::command]
pub async fn chat_send_text(
    state: State<'_, AppState>,
    message: String,
) -> Result<ChatReply, String> {
    let engine = state.engine.clone();
    let turn = engine
        .handle_text_turn(message)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ChatReply {
        answer: turn.answer,
    })
}

/// Reset the conversation history.
#[tauri::command]
pub async fn chat_reset(state: State<'_, AppState>) -> Result<(), String> {
    state.engine.reset();
    Ok(())
}

/// Snapshot of the current conversation history.
#[derive(Debug, Clone, Serialize)]
pub struct ChatHistoryEntry {
    /// `"user"` or `"assistant"`.
    pub role: String,
    /// Message content.
    pub content: String,
}

/// Return the current conversation history (without the system prompt).
#[tauri::command]
pub async fn chat_history(state: State<'_, AppState>) -> Result<Vec<ChatHistoryEntry>, String> {
    Ok(state
        .engine
        .history()
        .into_iter()
        .map(|m| ChatHistoryEntry {
            role: match m.role {
                andrea_llm::ChatRole::System => "system",
                andrea_llm::ChatRole::User => "user",
                andrea_llm::ChatRole::Assistant => "assistant",
            }
            .to_string(),
            content: m.content,
        })
        .collect())
}

// ---------- RAG ----------

/// One retrieved chunk shipped to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct RagHit {
    /// Stable chunk identifier.
    pub id: String,
    /// Citation string (e.g. `REAC V07 21/12/2022, CCP1, CP3`).
    pub citation: String,
    /// CCP code, if any.
    pub ccp: Option<String>,
    /// CP code, if any.
    pub cp: Option<String>,
    /// First N chars of the chunk text, useful for the UI to preview.
    pub snippet: String,
    /// Cosine similarity in `[-1.0, 1.0]`.
    pub score: f32,
}

const SNIPPET_CHARS: usize = 240;

/// Run a RAG search and return the top hits with citations.
#[tauri::command]
pub async fn rag_search(
    state: State<'_, AppState>,
    query: String,
    top_k: Option<usize>,
) -> Result<Vec<RagHit>, String> {
    let k = top_k.unwrap_or(4).clamp(1, 20);
    let hits = search(&*state.embedder, &*state.vector_store, &query, k)
        .await
        .map_err(|e| e.to_string())?;
    Ok(hits
        .into_iter()
        .map(|h| RagHit {
            snippet: snippet(&h.document.text),
            id: h.document.id,
            citation: h.document.citation,
            ccp: h.document.ccp,
            cp: h.document.cp,
            score: h.score,
        })
        .collect())
}

/// Number of chunks currently loaded in the vector store. Useful for the
/// UI to show "Référentiel chargé : N extraits".
#[tauri::command]
pub async fn rag_status(state: State<'_, AppState>) -> Result<usize, String> {
    Ok(state.vector_store.len().await)
}

fn snippet(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= SNIPPET_CHARS {
        collapsed
    } else {
        let mut out: String = collapsed.chars().take(SNIPPET_CHARS).collect();
        out.push('…');
        out
    }
}
