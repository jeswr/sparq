//! Match highlighting for resolved literal text.
//!
//! The caller supplies the literal lexical form after resolving a search hit's
//! dictionary id. Matching uses the default text-index analyzer, while returned
//! spans always refer to the original string's UTF-8 byte offsets.

use crate::tokenize::{tokenize_with, Analyzer};
use unicode_segmentation::UnicodeSegmentation;

const OPEN_MARK: &str = "<mark>";
const CLOSE_MARK: &str = "</mark>";

/// A highlighted window into a literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snippet {
    /// Matched half-open UTF-8 byte ranges in the complete source text.
    pub spans: Vec<(usize, usize)>,
    /// The contiguous source window with every matched token wrapped in
    /// `<mark>` and `</mark>`.
    pub rendered: String,
}

/// Highlights every case-insensitive occurrence of one analyzed query token.
///
/// Matching uses [`Analyzer::Unicode`], the default analyzer used by
/// [`crate::TextIndex`]. The context budget is measured in bytes on each side
/// of the outermost matches. A clipped leading or trailing word is removed,
/// and UTF-8 code points are never split. The returned spans remain absolute
/// ranges into `text`, rather than ranges relative to `rendered`.
///
/// Returns `None` when `text` is empty, when `query_token` does not analyze to
/// exactly one token, or when the token does not occur.
pub fn snippet(text: &str, query_token: &str, context_chars: usize) -> Option<Snippet> {
    if text.is_empty() {
        return None;
    }

    let analyzer = Analyzer::Unicode;
    let mut query_tokens = tokenize_with(query_token, analyzer);
    if query_tokens.len() != 1 {
        return None;
    }
    let query_token = query_tokens.pop().expect("length checked above");

    let spans: Vec<(usize, usize)> = text
        .unicode_word_indices()
        .filter_map(|(start, word)| {
            let analyzed = tokenize_with(word, analyzer);
            (analyzed.len() == 1 && analyzed[0] == query_token)
                .then_some((start, start + word.len()))
        })
        .collect();
    let &(first_start, _) = spans.first()?;
    let &(_, last_end) = spans.last()?;

    let window_start = window_start(text, first_start, context_chars);
    let window_end = window_end(text, last_end, context_chars);
    let marker_bytes = spans
        .len()
        .saturating_mul(OPEN_MARK.len() + CLOSE_MARK.len());
    let mut rendered = String::with_capacity(window_end - window_start + marker_bytes);
    let mut cursor = window_start;
    for &(start, end) in &spans {
        rendered.push_str(&text[cursor..start]);
        rendered.push_str(OPEN_MARK);
        rendered.push_str(&text[start..end]);
        rendered.push_str(CLOSE_MARK);
        cursor = end;
    }
    rendered.push_str(&text[cursor..window_end]);

    Some(Snippet { spans, rendered })
}

fn window_start(text: &str, first_start: usize, context_chars: usize) -> usize {
    let mut clipped = first_start.saturating_sub(context_chars);
    while !text.is_char_boundary(clipped) {
        clipped += 1;
    }

    text.unicode_word_indices()
        .map(|(start, _)| start)
        .find(|&start| start > clipped && start <= first_start)
        .unwrap_or(first_start)
}

fn window_end(text: &str, last_end: usize, context_chars: usize) -> usize {
    let mut clipped = last_end.saturating_add(context_chars).min(text.len());
    while !text.is_char_boundary(clipped) {
        clipped -= 1;
    }

    text.unicode_word_indices()
        .map(|(start, word)| start + word.len())
        .take_while(|&end| end <= clipped)
        .filter(|&end| end >= last_end)
        .last()
        .unwrap_or(last_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_exact_word_trimmed_context() {
        let result = snippet("the quick fox jumps", "fox", 10).expect("fox matches");
        assert_eq!(result.spans, [(10, 13)]);
        assert_eq!(result.rendered, "quick <mark>fox</mark> jumps");
    }

    #[test]
    fn marks_every_case_insensitive_occurrence() {
        let result = snippet("Fox fOX", "FOX", 0).expect("both tokens match");
        assert_eq!(result.spans, [(0, 3), (4, 7)]);
        assert_eq!(result.rendered, "<mark>Fox</mark> <mark>fOX</mark>");
    }

    #[test]
    fn rejects_empty_missing_or_non_single_tokens() {
        assert_eq!(snippet("", "fox", 10), None);
        assert_eq!(snippet("the quick fox", "wolf", 10), None);
        assert_eq!(snippet("the quick fox", "quick fox", 10), None);
        assert_eq!(snippet("the quick fox", "", 10), None);
    }

    #[test]
    fn multibyte_context_cut_stays_on_utf8_boundary() {
        let text = "é café fox tail";
        let result = snippet(text, "fox", 8).expect("fox matches");
        assert_eq!(result.spans, [(9, 12)]);
        assert_eq!(result.rendered, "café <mark>fox</mark> tail");
        for &(start, end) in &result.spans {
            assert!(text.is_char_boundary(start));
            assert!(text.is_char_boundary(end));
            assert_eq!(&text[start..end], "fox");
        }
    }
}
