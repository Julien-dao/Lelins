//! Onboarding wizard logic for ANDREA.
//!
//! Captures the apprenant's profile in a small finite-state machine, then
//! produces a [`Profile`] used downstream by `andrea-prompt::ProfileVars`
//! and the `profile` SQLite row. The UI consumes
//! [`OnboardingDraft::next_step`] to know which screen to render and
//! [`OnboardingDraft::is_complete`] to enable the "Commencer" button.
//!
//! Static data:
//!
//! - [`AvatarOption`] gallery via [`AVATAR_GALLERY`] (12 portraits, mixed
//!   age and origin per `docs/02-direction-artistique.md`).
//! - [`VoiceOption`] catalogue via [`VOICE_CATALOGUE`] (Piper FR voices).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod avatars;
mod profile;
mod validation;
mod voices;

pub use avatars::{AvatarGender, AvatarOption, AvatarOrigin, AVATAR_GALLERY};
pub use profile::{
    Address, Cadence, Goal, OnboardingDraft, OnboardingStep, Profile, StarterLevel, Theme,
};
pub use validation::{validate_email, validate_first_name, ValidationError};
pub use voices::{find_voice, VoiceGender, VoiceOption, VOICE_CATALOGUE};
