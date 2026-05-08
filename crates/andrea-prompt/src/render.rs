//! Minimal `{{ var }}` template engine.
//!
//! The grammar is small enough that pulling in `tera`/`handlebars` would be
//! overkill — and the templates are linted by `validate_template_for` so
//! typos surface at test time rather than at runtime.
//!
//! Syntax:
//!
//! - `{{ name }}` — interpolated. Whitespace inside braces is optional.
//! - `{{name}}` — also accepted.
//! - `{{ name }}` matched **only** if `name` resolves to a known variable;
//!   unknown names are left untouched (so a missing variable produces a
//!   visible `{{ name }}` in the output rather than a panic).

use std::collections::BTreeMap;
use thiserror::Error;

use crate::profile::ProfileVars;

/// The verbatim ANDREA Formateur v1 prompt, embedded at compile time.
pub const FORMATEUR_V1_TEMPLATE: &str =
    include_str!("../../../packages/prompts/andrea-formateur-v1.md");

/// Errors raised when rendering a template.
#[derive(Debug, Error)]
pub enum RenderError {
    /// One or more variables present in the template are not provided.
    #[error("missing template variable(s): {0:?}")]
    MissingVariables(Vec<String>),
}

/// Render the ANDREA Formateur v1 prompt with `profile` substitutions.
///
/// All variables required by the template must be present in `profile`;
/// callers ensure this by always passing a complete [`ProfileVars`] (use
/// [`ProfileVars::placeholder`] for incomplete profiles).
pub fn render_formateur_v1(profile: &ProfileVars) -> Result<String, RenderError> {
    let vars = profile_to_map(profile);
    render_template(FORMATEUR_V1_TEMPLATE, &vars)
}

/// Render an arbitrary template with the given variable map. Callers
/// outside ANDREA Formateur v1 use this for ad-hoc prompts.
pub fn render_template(
    template: &str,
    vars: &BTreeMap<&str, String>,
) -> Result<String, RenderError> {
    let mut out = String::with_capacity(template.len());
    let mut missing: Vec<String> = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume the second {
            let mut name = String::new();
            let mut closed = false;
            while let Some(ch) = chars.next() {
                if ch == '}' && chars.peek() == Some(&'}') {
                    chars.next(); // consume the second }
                    closed = true;
                    break;
                }
                name.push(ch);
            }
            if !closed {
                // Unterminated `{{` — emit literally, including what we ate.
                out.push_str("{{");
                out.push_str(&name);
                continue;
            }
            let key = name.trim();
            match vars.get(key) {
                Some(value) => out.push_str(value),
                None => {
                    missing.push(key.to_string());
                    out.push_str("{{ ");
                    out.push_str(key);
                    out.push_str(" }}");
                }
            }
        } else {
            out.push(c);
        }
    }

    if !missing.is_empty() {
        // Stable, deduplicated list for clear error messages.
        let mut dedup: Vec<String> = missing.into_iter().collect();
        dedup.sort();
        dedup.dedup();
        return Err(RenderError::MissingVariables(dedup));
    }
    Ok(out)
}

fn profile_to_map(p: &ProfileVars) -> BTreeMap<&str, String> {
    let mut m = BTreeMap::new();
    m.insert("formateur_nom", p.formateur_nom.clone());
    m.insert("prenom", p.prenom.clone());
    m.insert("appel", p.appel.clone());
    m.insert("tu_vous", p.tu_vous.clone());
    m.insert("accord_genre", p.accord_genre.clone());
    m.insert("contexte", p.contexte.clone());
    m.insert("niveau_depart", p.niveau_depart.clone());
    m.insert("objectif", p.objectif.clone());
    m.insert("cadence", p.cadence.clone());
    m.insert("date_epreuve", p.date_epreuve.clone());
    m.insert("progression_summary", p.progression_summary.clone());
    m.insert("session_summary", p.session_summary.clone());
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile_marie() -> ProfileVars {
        ProfileVars {
            formateur_nom: "ANDREA".to_string(),
            prenom: "Marie".to_string(),
            appel: "Marie".to_string(),
            tu_vous: "vous".to_string(),
            accord_genre: "votre formatrice".to_string(),
            contexte: "candidate VAE en exercice".to_string(),
            niveau_depart: "formatrice en exercice depuis 5 ans".to_string(),
            objectif: "titre complet".to_string(),
            cadence: "régulière, 3 heures par semaine".to_string(),
            date_epreuve: "fin novembre 2026".to_string(),
            progression_summary: "CP1 acquis, CP3 en cours".to_string(),
            session_summary: "Lors de notre dernier échange, nous avons abordé Bloom.".to_string(),
        }
    }

    #[test]
    fn render_substitutes_all_known_placeholders() {
        let mut vars = BTreeMap::new();
        vars.insert("name", "Marie".to_string());
        vars.insert("greeting", "Bonjour".to_string());
        let out = render_template("{{ greeting }} {{ name }} !", &vars).unwrap();
        assert_eq!(out, "Bonjour Marie !");
    }

    #[test]
    fn render_accepts_no_spaces() {
        let mut vars = BTreeMap::new();
        vars.insert("x", "42".to_string());
        let out = render_template("value={{x}}.", &vars).unwrap();
        assert_eq!(out, "value=42.");
    }

    #[test]
    fn render_returns_missing_variables() {
        let vars = BTreeMap::new();
        let err = render_template("{{ a }} and {{ b }}", &vars).unwrap_err();
        match err {
            RenderError::MissingVariables(v) => {
                assert_eq!(v, vec!["a".to_string(), "b".to_string()]);
            }
        }
    }

    #[test]
    fn render_leaves_unterminated_braces_untouched() {
        let vars = BTreeMap::new();
        // No closing `}}` — should be passed through literally.
        let out = render_template("text {{ unfinished", &vars).unwrap();
        assert!(out.contains("{{"));
        assert!(out.contains("unfinished"));
    }

    #[test]
    fn render_formateur_v1_substitutes_profile() {
        let p = profile_marie();
        let prompt = render_formateur_v1(&p).expect("render succeeds");
        assert!(prompt.contains("Marie"));
        assert!(prompt.contains("votre formatrice"));
        assert!(prompt.contains("formatrice en exercice depuis 5 ans"));
        assert!(prompt.contains("titre complet"));
        // No raw placeholders left.
        assert!(
            !prompt.contains("{{ "),
            "leftover placeholder in:\n{prompt}"
        );
    }

    #[test]
    fn render_formateur_v1_contains_garde_fous() {
        let prompt = render_formateur_v1(&profile_marie()).unwrap();
        // Hard rules from docs/03 must survive interpolation.
        assert!(prompt.contains("Ne JAMAIS inventer une référence"));
        assert!(prompt.contains("TOUJOURS citer la source"));
        assert!(prompt.contains("Ne JAMAIS te présenter comme certificateur"));
        assert!(prompt.contains("Ne JAMAIS rédiger le Dossier Professionnel"));
        assert!(prompt.contains("100 % en local"));
    }

    #[test]
    fn render_formateur_v1_contains_4_ccp_structure() {
        let prompt = render_formateur_v1(&profile_marie()).unwrap();
        assert!(prompt.contains("CCP1"));
        assert!(prompt.contains("CCP2"));
        assert!(prompt.contains("CCP3"));
        assert!(prompt.contains("CCP4"));
        assert!(prompt.contains("CP1"));
        assert!(prompt.contains("CP13"));
    }

    #[test]
    fn render_formateur_v1_contains_knowles_principles() {
        let prompt = render_formateur_v1(&profile_marie()).unwrap();
        assert!(prompt.contains("Knowles"));
        assert!(prompt.contains("Besoin de savoir"));
        assert!(prompt.contains("Concept de soi"));
        assert!(prompt.contains("Rôle de l'expérience"));
        assert!(prompt.contains("Volonté d'apprendre"));
        assert!(prompt.contains("Orientation vers la résolution"));
        assert!(prompt.contains("Motivation intrinsèque"));
    }

    #[test]
    fn render_formateur_v1_with_placeholder_profile_renders() {
        let p = ProfileVars::placeholder();
        let prompt = render_formateur_v1(&p).unwrap();
        assert!(prompt.contains("apprenant·e"));
        assert!(!prompt.contains("{{ "));
    }
}
