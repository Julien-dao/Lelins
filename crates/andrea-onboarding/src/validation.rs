//! Lightweight validators for onboarding inputs. No regex (avoids the
//! `regex` dependency for a tiny win); all rules are explicit and
//! deliberately conservative.

use thiserror::Error;

/// Errors raised by validation.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    /// Field name was not supplied at all.
    #[error("missing field `{0}`")]
    Missing(&'static str),
    /// First name is blank, too short, or too long.
    #[error("invalid first name: {0}")]
    InvalidFirstName(&'static str),
    /// Email does not pass our basic structural check.
    #[error("invalid email: {0}")]
    InvalidEmail(&'static str),
}

const MIN_NAME_LEN: usize = 1;
const MAX_NAME_LEN: usize = 60;
const MAX_EMAIL_LEN: usize = 254;

/// Validate a first name. Trims, then checks length and forbids characters
/// that are obviously not part of a personal name (control chars, `<`, `>`).
pub fn validate_first_name(raw: &str) -> Result<String, ValidationError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::InvalidFirstName("empty"));
    }
    let len = trimmed.chars().count();
    if len < MIN_NAME_LEN {
        return Err(ValidationError::InvalidFirstName("too short"));
    }
    if len > MAX_NAME_LEN {
        return Err(ValidationError::InvalidFirstName("too long"));
    }
    for c in trimmed.chars() {
        if c.is_control() || matches!(c, '<' | '>' | '\\' | '/' | '\t') {
            return Err(ValidationError::InvalidFirstName("contains forbidden char"));
        }
    }
    Ok(trimmed.to_string())
}

/// Validate an email. Conservative structural check: exactly one `@`,
/// non-empty local + domain parts, no whitespace, length cap.
pub fn validate_email(raw: &str) -> Result<String, ValidationError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::InvalidEmail("empty"));
    }
    if trimmed.len() > MAX_EMAIL_LEN {
        return Err(ValidationError::InvalidEmail("too long"));
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(ValidationError::InvalidEmail("contains whitespace"));
    }
    let at_count = trimmed.chars().filter(|c| *c == '@').count();
    if at_count != 1 {
        return Err(ValidationError::InvalidEmail("missing or extra '@'"));
    }
    let (local, domain) = trimmed.split_once('@').unwrap();
    if local.is_empty() || domain.is_empty() {
        return Err(ValidationError::InvalidEmail("empty local or domain"));
    }
    if !domain.contains('.') {
        return Err(ValidationError::InvalidEmail("domain has no TLD"));
    }
    if domain.starts_with('.') || domain.ends_with('.') {
        return Err(ValidationError::InvalidEmail(
            "domain starts or ends with dot",
        ));
    }
    Ok(trimmed.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_basic_cases() {
        assert_eq!(validate_first_name("Marie").unwrap(), "Marie");
        assert_eq!(
            validate_first_name("  Jean-Pierre  ").unwrap(),
            "Jean-Pierre"
        );
    }

    #[test]
    fn name_rejects_empty() {
        assert!(matches!(
            validate_first_name(""),
            Err(ValidationError::InvalidFirstName(_))
        ));
        assert!(matches!(
            validate_first_name("   "),
            Err(ValidationError::InvalidFirstName(_))
        ));
    }

    #[test]
    fn name_rejects_too_long() {
        let long = "a".repeat(61);
        assert!(matches!(
            validate_first_name(&long),
            Err(ValidationError::InvalidFirstName("too long"))
        ));
    }

    #[test]
    fn name_rejects_html_chars() {
        assert!(matches!(
            validate_first_name("<script>"),
            Err(ValidationError::InvalidFirstName(_))
        ));
    }

    #[test]
    fn email_basic_cases() {
        assert_eq!(
            validate_email("Julien@Andrea-formation.fr").unwrap(),
            "julien@andrea-formation.fr"
        );
    }

    #[test]
    fn email_rejects_no_at() {
        assert!(validate_email("plain.text").is_err());
    }

    #[test]
    fn email_rejects_two_at() {
        assert!(validate_email("a@b@c.fr").is_err());
    }

    #[test]
    fn email_rejects_no_tld() {
        assert!(validate_email("a@b").is_err());
    }

    #[test]
    fn email_rejects_whitespace() {
        assert!(validate_email("a b@c.fr").is_err());
    }

    #[test]
    fn email_rejects_empty_local_or_domain() {
        assert!(validate_email("@x.fr").is_err());
        assert!(validate_email("a@").is_err());
        assert!(validate_email("a@.fr").is_err());
        assert!(validate_email("a@x.").is_err());
    }
}
