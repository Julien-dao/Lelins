//! ANDREA prompt rendering and pedagogical evaluation set.
//!
//! Two responsibilities:
//!
//! 1. Load the **ANDREA Formateur v1** system prompt template from
//!    `packages/prompts/andrea-formateur-v1.md` (embedded at compile time)
//!    and render it with the apprenant's profile via [`render_formateur_v1`].
//! 2. Expose a **30-scenario pedagogical evaluation set** (cf. `docs/03`)
//!    used to validate that the rendered prompt + the LLM behaviour stay
//!    aligned with ANDREA's posture (Knowles, garde-fous, citations).
//!
//! Note: the LLM-as-judge runner that grades a model against the scenarios
//! lives in a separate binary built only when Ollama is reachable. This
//! crate ships the *data* and the *renderer* — both fully testable on
//! Linux without any model.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod profile;
mod render;
mod scenarios;

pub use profile::ProfileVars;
pub use render::{render_formateur_v1, render_template, RenderError, FORMATEUR_V1_TEMPLATE};
pub use scenarios::{Scenario, ScenarioCategory, ScenarioCheck, ANDREA_SCENARIOS};
