//! ANDREA desktop runtime — Tauri shell wiring the domain crates together.
//!
//! Most of the actual work lives in the `andrea-*` crates. This crate
//! exposes Tauri commands that the React frontend invokes via
//! `@tauri-apps/api/core::invoke`.

mod commands;
mod state;

use tauri::Manager;

use crate::state::AppState;

/// Entry point invoked from `main.rs` or as a mobile lib entry.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .manage(AppState::from_env())
        .setup(|app| {
            // Resolve the per-user data directory and run migrations.
            let app_data = app
                .path()
                .app_data_dir()
                .expect("platform must provide an app data dir");
            std::fs::create_dir_all(&app_data).ok();
            let db_path = app_data.join("andrea.db");
            let _conn = andrea_db::open_with_migrations(&db_path, |progress| {
                eprintln!("[migrations] {progress:?}");
            })?;

            // Ingest the bootstrap référentiel chunks. Done in the setup
            // hook (off the main thread via spawn) so the first user turn
            // already has retrieval available. Errors are logged but do
            // not block startup — RAG augmentation degrades gracefully.
            let state: tauri::State<AppState> = app.state();
            let state_clone = AppState {
                engine: state.engine.clone(),
                embedder: state.embedder.clone(),
                vector_store: state.vector_store.clone(),
                license_secret: state.license_secret.clone(),
            };
            tauri::async_runtime::spawn(async move {
                if let Err(e) = state_clone.ingest_bootstrap_corpus().await {
                    eprintln!("[rag] bootstrap ingestion failed: {e}");
                } else {
                    eprintln!(
                        "[rag] bootstrap corpus ingested ({} chunks)",
                        state_clone.vector_store.len().await
                    );
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::license_validate,
            commands::license_info,
            commands::hardware_profile,
            commands::recommended_model,
            commands::minimum_ram_gb,
            commands::chat_send_text,
            commands::chat_reset,
            commands::chat_history,
            commands::rag_search,
            commands::rag_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ANDREA desktop");
}
