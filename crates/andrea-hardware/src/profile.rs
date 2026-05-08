//! Hardware profile and model-selection algorithm.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Snapshot of the host hardware at app startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    /// Total physical RAM in gibibytes (1024³ bytes), rounded down.
    pub total_ram_gb: u32,
    /// Number of logical CPUs.
    pub logical_cpus: u32,
    /// CPU brand string when reported by the OS.
    pub cpu_brand: Option<String>,
    /// Target architecture (`x86_64`, `aarch64`, …).
    pub arch: String,
    /// Operating system (`macos`, `windows`, `linux`, …).
    pub os: String,
}

/// Tier of LLM ANDREA will load by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    /// 24B class — Mistral Small 3.2.
    Premium,
    /// 12B class — Mistral Nemo.
    Standard,
    /// ~3-4B class — Phi-4-mini.
    Light,
}

/// Concrete model recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelChoice {
    /// Tier this choice belongs to.
    pub tier: ModelTier,
    /// Ollama model identifier (e.g. `mistral-small3.2:24b`).
    pub ollama_id: &'static str,
    /// Human-readable display name.
    pub display_name: &'static str,
    /// Approximate disk footprint in gibibytes (download size + on-disk).
    pub disk_size_gb: u32,
    /// Approximate runtime RAM consumption.
    pub runtime_ram_gb: u32,
    /// Brief description shown to the user during onboarding.
    pub blurb: &'static str,
}

/// Minimum RAM supported. Below this, ANDREA refuses to install a model.
pub const MIN_SUPPORTED_RAM_GB: u32 = 8;

const PREMIUM: ModelChoice = ModelChoice {
    tier: ModelTier::Premium,
    ollama_id: "mistral-small3.2:24b",
    display_name: "Mistral Small 3.2 (24 milliards de paramètres)",
    disk_size_gb: 14,
    runtime_ram_gb: 16,
    blurb: "La meilleure expérience pédagogique en français — recommandé à partir de 24 Go de RAM.",
};

const STANDARD: ModelChoice = ModelChoice {
    tier: ModelTier::Standard,
    ollama_id: "mistral-nemo:12b",
    display_name: "Mistral Nemo (12 milliards de paramètres)",
    disk_size_gb: 7,
    runtime_ram_gb: 9,
    blurb: "Excellent compromis qualité / mémoire pour les machines de 12 à 24 Go.",
};

const LIGHT: ModelChoice = ModelChoice {
    tier: ModelTier::Light,
    ollama_id: "phi4-mini:3.8b",
    display_name: "Phi-4-mini (3,8 milliards de paramètres)",
    disk_size_gb: 3,
    runtime_ram_gb: 5,
    blurb:
        "Mode léger pour les MacBook Air 8 Go et les PC modestes. Pédagogie possible, finesse réduite.",
};

/// Errors raised when no model fits.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SelectionError {
    /// Total RAM is below [`MIN_SUPPORTED_RAM_GB`].
    #[error("RAM insuffisante : {total_ram_gb} Go détectés, {min} Go minimum requis")]
    InsufficientRam {
        /// Detected RAM.
        total_ram_gb: u32,
        /// Minimum required RAM.
        min: u32,
    },
}

impl HardwareProfile {
    /// Pick the best [`ModelChoice`] for this hardware.
    pub fn select_model(&self) -> Result<ModelChoice, SelectionError> {
        select_model_for_ram(self.total_ram_gb)
    }

    /// Whether the profile reports an Apple Silicon GPU we can rely on.
    pub fn is_apple_silicon(&self) -> bool {
        self.os == "macos" && self.arch == "aarch64"
    }
}

/// Pure function used by the selection logic. Exposed for unit tests.
pub fn select_model_for_ram(total_ram_gb: u32) -> Result<ModelChoice, SelectionError> {
    if total_ram_gb < MIN_SUPPORTED_RAM_GB {
        return Err(SelectionError::InsufficientRam {
            total_ram_gb,
            min: MIN_SUPPORTED_RAM_GB,
        });
    }
    if total_ram_gb >= 24 {
        Ok(PREMIUM)
    } else if total_ram_gb >= 12 {
        Ok(STANDARD)
    } else {
        Ok(LIGHT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_with(ram_gb: u32) -> HardwareProfile {
        HardwareProfile {
            total_ram_gb: ram_gb,
            logical_cpus: 8,
            cpu_brand: None,
            arch: "x86_64".to_string(),
            os: "linux".to_string(),
        }
    }

    #[test]
    fn premium_at_or_above_24_gb() {
        for ram in [24, 32, 48, 64, 128] {
            let m = profile_with(ram).select_model().unwrap();
            assert_eq!(m.tier, ModelTier::Premium, "ram = {ram}");
        }
    }

    #[test]
    fn standard_between_12_and_24_gb() {
        for ram in [12, 14, 16, 23] {
            let m = profile_with(ram).select_model().unwrap();
            assert_eq!(m.tier, ModelTier::Standard, "ram = {ram}");
        }
    }

    #[test]
    fn light_between_8_and_12_gb() {
        for ram in [8, 9, 10, 11] {
            let m = profile_with(ram).select_model().unwrap();
            assert_eq!(m.tier, ModelTier::Light, "ram = {ram}");
        }
    }

    #[test]
    fn refused_below_minimum() {
        for ram in [0, 1, 4, 7] {
            let err = profile_with(ram).select_model().unwrap_err();
            assert_eq!(
                err,
                SelectionError::InsufficientRam {
                    total_ram_gb: ram,
                    min: MIN_SUPPORTED_RAM_GB
                }
            );
        }
    }

    #[test]
    fn apple_silicon_detection() {
        let mut p = profile_with(16);
        p.os = "macos".to_string();
        p.arch = "aarch64".to_string();
        assert!(p.is_apple_silicon());

        p.arch = "x86_64".to_string();
        assert!(!p.is_apple_silicon());

        p.arch = "aarch64".to_string();
        p.os = "linux".to_string();
        assert!(!p.is_apple_silicon());
    }

    #[test]
    fn model_choice_is_serializable() {
        let m = select_model_for_ram(16).unwrap();
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("standard"));
        assert!(json.contains("mistral-nemo"));
    }

    #[test]
    fn boundary_24_is_premium_not_standard() {
        let m = profile_with(24).select_model().unwrap();
        assert_eq!(m.tier, ModelTier::Premium);
    }

    #[test]
    fn boundary_12_is_standard_not_light() {
        let m = profile_with(12).select_model().unwrap();
        assert_eq!(m.tier, ModelTier::Standard);
    }

    #[test]
    fn boundary_8_is_light_not_refused() {
        let m = profile_with(8).select_model().unwrap();
        assert_eq!(m.tier, ModelTier::Light);
    }
}
