//! Manifest of downloadable assets bundled with each ANDREA release.

use serde::{Deserialize, Serialize};

/// Category of an asset; used for UX grouping ("Téléchargement du moteur LLM…").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetCategory {
    /// Ollama LLM model (one of the three tiers).
    Llm,
    /// Whisper GGML model for speech-to-text.
    Stt,
    /// Piper ONNX voice for text-to-speech.
    Tts,
    /// Pre-chunked référentiel FPA + JSON metadata.
    Referentiel,
}

/// Specification for a single downloadable asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSpec {
    /// Stable identifier used by the application to look up local paths.
    pub id: String,
    /// Category for UX grouping.
    pub category: AssetCategory,
    /// Human-friendly display name.
    pub display_name: String,
    /// HTTPS URL to download from. May be a release artifact or a CDN.
    pub url: String,
    /// Expected SHA-256 of the final file (lowercase hex).
    pub sha256_hex: String,
    /// Expected file size in bytes (informational; checksum is authoritative).
    pub size_bytes: u64,
}

/// Top-level manifest bundled with each release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Schema version of the manifest itself.
    pub schema_version: u32,
    /// Application version that produced the manifest.
    pub app_version: String,
    /// All assets ANDREA may need to download.
    pub assets: Vec<AssetSpec>,
}

impl Manifest {
    /// Find an asset by its identifier.
    pub fn get(&self, id: &str) -> Option<&AssetSpec> {
        self.assets.iter().find(|a| a.id == id)
    }

    /// Enumerate assets of a given category.
    pub fn by_category(&self, cat: AssetCategory) -> impl Iterator<Item = &AssetSpec> {
        self.assets.iter().filter(move |a| a.category == cat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrips_through_json() {
        let m = Manifest {
            schema_version: 1,
            app_version: "0.1.0".to_string(),
            assets: vec![AssetSpec {
                id: "mistral-nemo-12b".to_string(),
                category: AssetCategory::Llm,
                display_name: "Mistral Nemo (12B)".to_string(),
                url: "https://example.com/mistral-nemo-12b.bin".to_string(),
                sha256_hex: "0".repeat(64),
                size_bytes: 7_100_000_000,
            }],
        };
        let json = serde_json::to_string_pretty(&m).unwrap();
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.schema_version, 1);
        assert_eq!(back.assets.len(), 1);
    }

    #[test]
    fn lookup_by_id_and_category() {
        let m = Manifest {
            schema_version: 1,
            app_version: "0.1.0".to_string(),
            assets: vec![
                AssetSpec {
                    id: "a".to_string(),
                    category: AssetCategory::Llm,
                    display_name: "A".to_string(),
                    url: "https://x/a".to_string(),
                    sha256_hex: "0".repeat(64),
                    size_bytes: 1,
                },
                AssetSpec {
                    id: "b".to_string(),
                    category: AssetCategory::Stt,
                    display_name: "B".to_string(),
                    url: "https://x/b".to_string(),
                    sha256_hex: "0".repeat(64),
                    size_bytes: 2,
                },
            ],
        };
        assert_eq!(m.get("a").unwrap().display_name, "A");
        assert!(m.get("missing").is_none());
        assert_eq!(m.by_category(AssetCategory::Stt).count(), 1);
    }
}
