//! Shared runtime state for ANDREA Tauri commands.
//!
//! Held inside `tauri::State<AppState>`. Currently a stub — subsequent
//! steps (LLM, RAG, conversation) will hang fields off this struct.

#[derive(Default)]
pub struct AppState {
    /// Reserved: the embedded server secret used by `andrea-license`.
    /// In production builds this is injected at compile time via
    /// `env!("ANDREA_LICENSE_SECRET")`. In dev builds a placeholder is used.
    pub license_secret: Vec<u8>,
}

impl AppState {
    /// Build a state with the production-embedded secret (if any) or a
    /// dev placeholder.
    pub fn from_env() -> Self {
        let secret = option_env!("ANDREA_LICENSE_SECRET")
            .map(|s| s.as_bytes().to_vec())
            .unwrap_or_else(|| b"andrea-dev-placeholder-secret-do-not-ship".to_vec());
        Self {
            license_secret: secret,
        }
    }
}
