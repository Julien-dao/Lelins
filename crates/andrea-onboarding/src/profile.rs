//! Apprenant profile and onboarding draft state machine.

use serde::{Deserialize, Serialize};

use crate::validation::{validate_email, validate_first_name, ValidationError};

/// Form of address chosen by the apprenant during onboarding.
///
/// Per `docs/08-decisions.md`, the user is **always asked** explicitly —
/// there is no silent default. The serialized form maps directly onto the
/// `profile.tutoiement` SQLite column (0 = vous, 1 = tu).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Address {
    /// Tutoiement (`tu`).
    Tu,
    /// Vouvoiement (`vous`).
    Vous,
}

impl Address {
    /// String form used inside the rendered ANDREA Formateur v1 prompt.
    pub fn as_prompt_str(self) -> &'static str {
        match self {
            Address::Tu => "tu",
            Address::Vous => "vous",
        }
    }
}

/// Where the apprenant is starting from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StarterLevel {
    /// Starting from scratch.
    Beginner,
    /// Has practiced informally without certification.
    PracticalNoFormal,
    /// Currently practicing as a formateur.
    InService,
    /// Pivoting from another profession.
    Reconversion,
}

impl StarterLevel {
    /// Human-readable label inserted into the rendered prompt.
    pub fn as_prompt_str(self) -> &'static str {
        match self {
            StarterLevel::Beginner => "débutant·e total·e",
            StarterLevel::PracticalNoFormal => "expérience pratique sans formation",
            StarterLevel::InService => "formateur·rice en exercice",
            StarterLevel::Reconversion => "en reconversion vers la formation",
        }
    }
}

/// What the apprenant is aiming for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Goal {
    /// Full title.
    FullTitle,
    /// CCP1 only.
    Ccp1Only,
    /// CCP2 only.
    Ccp2Only,
    /// CCP3 only.
    Ccp3Only,
    /// CCP4 only.
    Ccp4Only,
    /// Skill-up without aiming at the certification.
    SkillUp,
}

impl Goal {
    /// Human-readable label for the prompt.
    pub fn as_prompt_str(self) -> &'static str {
        match self {
            Goal::FullTitle => "titre complet",
            Goal::Ccp1Only => "CCP1 seul",
            Goal::Ccp2Only => "CCP2 seul",
            Goal::Ccp3Only => "CCP3 seul",
            Goal::Ccp4Only => "CCP4 seul",
            Goal::SkillUp => "montée en compétences sans certification",
        }
    }
}

/// Pace at which the apprenant wants to study.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cadence {
    /// 10 h+ per week.
    Intensive,
    /// 3-5 h per week.
    Regular,
    /// As they go.
    Free,
}

impl Cadence {
    /// Human-readable label for the prompt.
    pub fn as_prompt_str(self) -> &'static str {
        match self {
            Cadence::Intensive => "intensive (≥ 10 h par semaine)",
            Cadence::Regular => "régulière (3 à 5 h par semaine)",
            Cadence::Free => "libre, à mon rythme",
        }
    }
}

/// UI theme stored on the profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Follow the system preference (default on Mac & Win).
    System,
    /// Always dark.
    Dark,
    /// Always light.
    Light,
}

/// Final profile produced by the onboarding wizard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// First name.
    pub prenom: String,
    /// How to address the apprenant by name (defaults to `prenom`).
    pub appel: String,
    /// Verified email.
    pub email: String,
    /// Tu / vous.
    pub address: Address,
    /// Starting level.
    pub starter_level: StarterLevel,
    /// Goal.
    pub goal: Goal,
    /// Cadence.
    pub cadence: Cadence,
    /// Optional exam date in `YYYY-MM-DD`.
    pub date_epreuve: Option<String>,
    /// Selected avatar id (cf. [`crate::AVATAR_GALLERY`]) or `None` for
    /// the user-uploaded image case.
    pub avatar_id: Option<String>,
    /// Selected voice id (cf. [`crate::VOICE_CATALOGUE`]).
    pub voice_id: String,
    /// Display name of the formateur (defaults to `ANDREA`).
    pub formateur_nom: String,
    /// Theme preference.
    pub theme: Theme,
}

/// Discrete steps of the onboarding wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStep {
    /// Splash + intro.
    Welcome,
    /// Capture license key.
    LicenseKey,
    /// Capture identity (prénom, email, address).
    Identity,
    /// Capture context (starter level, goal, cadence, exam date).
    Context,
    /// Choose avatar.
    Avatar,
    /// Choose voice.
    Voice,
    /// Confirm and download models.
    Confirm,
    /// First guided session by ANDREA.
    FirstSession,
}

/// Mutable draft used while filling the wizard.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnboardingDraft {
    /// Captured first name (validated by [`validate_first_name`]).
    pub prenom: Option<String>,
    /// Apprenant's chosen form of address.
    pub appel: Option<String>,
    /// Email (validated by [`validate_email`]).
    pub email: Option<String>,
    /// Form of address (`tu`/`vous`).
    pub address: Option<Address>,
    /// Starting level.
    pub starter_level: Option<StarterLevel>,
    /// Stated goal.
    pub goal: Option<Goal>,
    /// Wanted cadence.
    pub cadence: Option<Cadence>,
    /// Exam date.
    pub date_epreuve: Option<String>,
    /// Selected avatar id.
    pub avatar_id: Option<String>,
    /// Selected voice id.
    pub voice_id: Option<String>,
    /// Custom formateur name (default: "ANDREA").
    pub formateur_nom: Option<String>,
    /// Theme preference.
    pub theme: Option<Theme>,
}

impl OnboardingDraft {
    /// Empty draft.
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute the next step the UI should display.
    pub fn next_step(&self) -> OnboardingStep {
        if self.prenom.is_none() || self.email.is_none() || self.address.is_none() {
            return OnboardingStep::Identity;
        }
        if self.starter_level.is_none() || self.goal.is_none() || self.cadence.is_none() {
            return OnboardingStep::Context;
        }
        if self.avatar_id.is_none() {
            return OnboardingStep::Avatar;
        }
        if self.voice_id.is_none() {
            return OnboardingStep::Voice;
        }
        OnboardingStep::Confirm
    }

    /// `true` once every required field is set.
    pub fn is_complete(&self) -> bool {
        matches!(self.next_step(), OnboardingStep::Confirm)
    }

    /// Convert into a [`Profile`], validating along the way.
    pub fn into_profile(self) -> Result<Profile, ValidationError> {
        let prenom = self.prenom.ok_or(ValidationError::Missing("prenom"))?;
        validate_first_name(&prenom)?;
        let email = self.email.ok_or(ValidationError::Missing("email"))?;
        validate_email(&email)?;
        let appel = self
            .appel
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| prenom.clone());
        Ok(Profile {
            prenom,
            appel,
            email,
            address: self.address.ok_or(ValidationError::Missing("address"))?,
            starter_level: self
                .starter_level
                .ok_or(ValidationError::Missing("starter_level"))?,
            goal: self.goal.ok_or(ValidationError::Missing("goal"))?,
            cadence: self.cadence.ok_or(ValidationError::Missing("cadence"))?,
            date_epreuve: self.date_epreuve,
            avatar_id: self.avatar_id,
            voice_id: self.voice_id.ok_or(ValidationError::Missing("voice_id"))?,
            formateur_nom: self
                .formateur_nom
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "ANDREA".to_string()),
            theme: self.theme.unwrap_or(Theme::System),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_draft() -> OnboardingDraft {
        OnboardingDraft {
            prenom: Some("Marie".to_string()),
            appel: Some("Marie".to_string()),
            email: Some("marie@example.com".to_string()),
            address: Some(Address::Vous),
            starter_level: Some(StarterLevel::InService),
            goal: Some(Goal::FullTitle),
            cadence: Some(Cadence::Regular),
            date_epreuve: Some("2026-11-25".to_string()),
            avatar_id: Some("femme-creole-45".to_string()),
            voice_id: Some("siwis".to_string()),
            formateur_nom: None,
            theme: Some(Theme::Dark),
        }
    }

    #[test]
    fn next_step_advances_through_states() {
        let mut d = OnboardingDraft::new();
        assert_eq!(d.next_step(), OnboardingStep::Identity);
        d.prenom = Some("Marie".to_string());
        d.email = Some("m@x.fr".to_string());
        d.address = Some(Address::Vous);
        assert_eq!(d.next_step(), OnboardingStep::Context);
        d.starter_level = Some(StarterLevel::Beginner);
        d.goal = Some(Goal::Ccp1Only);
        d.cadence = Some(Cadence::Free);
        assert_eq!(d.next_step(), OnboardingStep::Avatar);
        d.avatar_id = Some("femme-metropolitaine-30".to_string());
        assert_eq!(d.next_step(), OnboardingStep::Voice);
        d.voice_id = Some("siwis".to_string());
        assert_eq!(d.next_step(), OnboardingStep::Confirm);
        assert!(d.is_complete());
    }

    #[test]
    fn into_profile_default_appel_is_prenom() {
        let mut d = complete_draft();
        d.appel = None;
        let p = d.into_profile().unwrap();
        assert_eq!(p.appel, "Marie");
    }

    #[test]
    fn into_profile_default_formateur_name_is_andrea() {
        let p = complete_draft().into_profile().unwrap();
        assert_eq!(p.formateur_nom, "ANDREA");
    }

    #[test]
    fn into_profile_rejects_missing_fields() {
        let mut d = complete_draft();
        d.email = None;
        let err = d.into_profile().unwrap_err();
        assert!(matches!(err, ValidationError::Missing("email")));
    }

    #[test]
    fn into_profile_rejects_bad_email() {
        let mut d = complete_draft();
        d.email = Some("not-an-email".to_string());
        let err = d.into_profile().unwrap_err();
        assert!(matches!(err, ValidationError::InvalidEmail(_)));
    }

    #[test]
    fn address_prompt_strings() {
        assert_eq!(Address::Tu.as_prompt_str(), "tu");
        assert_eq!(Address::Vous.as_prompt_str(), "vous");
    }

    #[test]
    fn enum_prompt_strings_are_french() {
        assert!(StarterLevel::Beginner.as_prompt_str().contains("débutant"));
        assert_eq!(Goal::FullTitle.as_prompt_str(), "titre complet");
        assert!(Cadence::Regular.as_prompt_str().contains("régulière"));
    }
}
