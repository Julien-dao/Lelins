//! Sentence splitter used to chunk ANDREA's streamed answer for sentence-by-
//! sentence TTS synthesis.
//!
//! Implementation is intentionally simple — full sentence segmentation in
//! French is hard (acronyms, abbreviations, decimals). The split rule:
//!
//! - cut after `.`, `!`, `?`, `…` followed by whitespace or end-of-string,
//! - never cut inside an obvious abbreviation (`M.`, `Mme.`, `etc.`, `cf.`,
//!   `c-à-d.`, `i.e.`, `e.g.`, `Dr.`, `Pr.`, decimals like `3.5`, `0.3`),
//! - never emit empty sentences.

/// Split `text` into sentences for sentence-by-sentence TTS.
///
/// Trailing punctuation is kept with the preceding sentence. Whitespace
/// between sentences is dropped.
pub fn split_into_sentences(text: &str) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        current.push(c);
        if matches!(c, '.' | '!' | '?' | '…') {
            // Look ahead: must be followed by whitespace or end.
            let followed_by_space = chars
                .get(i + 1)
                .map(|nc| nc.is_whitespace())
                .unwrap_or(true);
            if !followed_by_space {
                continue;
            }
            // Avoid splitting inside common abbreviations or decimals.
            if is_decimal_dot(&chars, i) || is_abbreviation_dot(&current) {
                continue;
            }
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                out.push(trimmed);
            }
            current.clear();
        }
    }
    let trailing = current.trim();
    if !trailing.is_empty() {
        out.push(trailing.to_string());
    }
    out
}

fn is_decimal_dot(chars: &[char], dot_idx: usize) -> bool {
    if dot_idx == 0 {
        return false;
    }
    let prev = chars[dot_idx - 1];
    let next = chars.get(dot_idx + 1);
    prev.is_ascii_digit() && next.is_some_and(|c| c.is_ascii_digit())
}

fn is_abbreviation_dot(current: &str) -> bool {
    // Look at the last "word" ending in this character.
    let trimmed_end = current.trim_end_matches(['.', '!', '?', '…']);
    let last_word = trimmed_end
        .rsplit_once(|c: char| c.is_whitespace())
        .map(|(_, w)| w)
        .unwrap_or(trimmed_end);
    let lowered = last_word.to_lowercase();
    matches!(
        lowered.as_str(),
        "m" | "mme"
            | "mlle"
            | "dr"
            | "pr"
            | "etc"
            | "cf"
            | "c-à-d"
            | "p.ex"
            | "i.e"
            | "e.g"
            | "vs"
            | "no"
            | "n°"
            | "av"
            | "j.-c"
            | "fig"
            | "art"
            | "p"
            | "pp"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_basic_french() {
        let out = split_into_sentences("Bonjour Marie. Comment allez-vous ? Êtes-vous prête !");
        assert_eq!(
            out,
            vec![
                "Bonjour Marie.".to_string(),
                "Comment allez-vous ?".to_string(),
                "Êtes-vous prête !".to_string(),
            ]
        );
    }

    #[test]
    fn empty_returns_empty() {
        assert!(split_into_sentences("").is_empty());
        assert!(split_into_sentences("   \n\t").is_empty());
    }

    #[test]
    fn does_not_split_decimals() {
        let out = split_into_sentences("La température est de 18.5 degrés. Voilà.");
        assert_eq!(out.len(), 2);
        assert!(out[0].contains("18.5"));
    }

    #[test]
    fn does_not_split_on_common_abbreviations() {
        let out = split_into_sentences("Voir M. Dupont, Mme. Martin et Dr. Smith. Suite ici.");
        assert_eq!(out.len(), 2, "got {out:?}");
        assert!(out[0].contains("Dr. Smith"));
    }

    #[test]
    fn preserves_terminal_punctuation() {
        let out = split_into_sentences("Première. Deuxième !");
        assert_eq!(out, vec!["Première.", "Deuxième !"]);
    }

    #[test]
    fn no_trailing_punctuation_still_emitted() {
        let out = split_into_sentences("phrase sans point final");
        assert_eq!(out, vec!["phrase sans point final"]);
    }

    #[test]
    fn ellipsis_counts_as_terminator() {
        let out = split_into_sentences("Hum… Voyons.");
        assert_eq!(out, vec!["Hum…", "Voyons."]);
    }
}
