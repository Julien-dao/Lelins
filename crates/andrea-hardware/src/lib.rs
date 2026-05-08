//! Hardware introspection and automatic model selection for ANDREA.
//!
//! ANDREA targets a wide range of consumer hardware. Per `docs/08-decisions.md`
//! we ship a **single application** with a model selection algorithm that
//! picks the best LLM the user's machine can run comfortably:
//!
//! | RAM      | Recommended LLM            | Quantization |
//! |----------|----------------------------|--------------|
//! | ≥ 24 Go  | Mistral Small 3.2 (24B)    | Q4_K_M       |
//! | 12 – 24  | Mistral Nemo (12B)         | Q4_K_M       |
//! |  8 – 12  | Phi-4-mini (3.8B)          | Q4_K_M       |
//! | < 8      | refused with a clear error |              |
//!
//! The user can override this in **Paramètres → Avancé** in a later step.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod profile;

pub use profile::{HardwareProfile, ModelChoice, ModelTier, SelectionError, MIN_SUPPORTED_RAM_GB};

use sysinfo::System;

/// Detect the current hardware profile.
///
/// Reads memory, CPU brand, and core count via `sysinfo`. Allocates a
/// `System` struct that `refresh_memory` updates — does not run a full
/// `refresh_all`, which would be wasteful here.
pub fn detect() -> HardwareProfile {
    let mut sys = System::new();
    sys.refresh_memory();
    let total_ram_bytes = sys.total_memory();
    // sysinfo returns bytes since 0.30; convert to GiB rounded down.
    let total_ram_gb = (total_ram_bytes / 1024 / 1024 / 1024) as u32;

    sys.refresh_cpu_all();
    let logical_cpus = sys.cpus().len() as u32;
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty());
    let arch = std::env::consts::ARCH.to_string();
    let os = std::env::consts::OS.to_string();

    HardwareProfile {
        total_ram_gb,
        logical_cpus,
        cpu_brand,
        arch,
        os,
    }
}

/// Convenience: detect hardware and pick a model in one call.
pub fn auto_select() -> Result<ModelChoice, SelectionError> {
    detect().select_model()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_a_plausible_profile() {
        let p = detect();
        // We are on Linux x86_64 in CI but the test must work on any host.
        assert!(p.logical_cpus >= 1, "got {p:?}");
        assert!(!p.os.is_empty());
        assert!(!p.arch.is_empty());
        // Memory may legitimately be 0 inside very stripped containers,
        // but real machines always have at least 1 GiB.
        // (We just assert the field is exposed.)
        let _ = p.total_ram_gb;
    }
}
