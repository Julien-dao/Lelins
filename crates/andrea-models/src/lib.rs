//! Resumable, checksummed downloader for ANDREA assets.
//!
//! ANDREA bootstraps several gigabytes of assets at first launch:
//!
//! - The Ollama LLM model picked by [`andrea_hardware`] (4 to 14 GB).
//! - One Whisper GGML model for STT (~466 MB for `small`).
//! - A few Piper voices in ONNX (~60 MB each).
//! - The pre-chunked référentiel FPA (a few MB).
//!
//! This crate provides a generic streaming downloader that:
//!
//! - resumes interrupted downloads via the `Range` HTTP header,
//! - verifies the resulting file against a manifest-supplied SHA-256,
//! - reports progress incrementally so the UI can render a progress bar.
//!
//! The actual list of assets is loaded from a JSON manifest bundled in the
//! desktop binary; this crate just consumes a [`AssetSpec`] at a time.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod download;
mod manifest;

pub use download::{download_with_resume, DownloadError, DownloadProgress};
pub use manifest::{AssetCategory, AssetSpec, Manifest};
