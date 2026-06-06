//! Unicode-aware text wrapping and cowsay-style speech bubbles.
//!
//! Widths are computed via [`unicode_width`] so that umlauts, emoji and
//! CJK characters do not shift the frames.

use unicode_width::UnicodeWidthStr;

/// Display width of a string in terminal columns.
#[must_use]
pub fn display_width(s: &str) -> usize {
    UnicodeWidthStr::width(s)
}

/// Display width excluding ANSI escape sequences (e.g. colour codes).
///
/// Needed to measure the *visible* width of already-coloured lines – for
/// example so the in-place redraw knows whether a line wraps.
#[must_use]
pub fn visible_width(s: &str) -> usize {
    let mut plain = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip the CSI sequence `ESC [ … <final 0x40..=0x7e>`.
            if chars.clone().next() == Some('[') {
                chars.next();
                for cc in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&cc) {
                        break;
                    }
                }
            } else {
                // Other escape sequence: discard the next character.
                chars.next();
            }
        } else {
            plain.push(c);
        }
    }
    display_width(&plain)
}

/// Wraps `text` at word boundaries into lines of at most `width` columns.
///
/// Individual, overly long words may exceed the width (they are not hard
/// broken). The result always contains at least one line.
///
/// ```
/// use rubberduck_cli::ui::text::wrap;
/// let lines = wrap("eins zwei drei", 9);
/// assert_eq!(lines, vec!["eins zwei".to_string(), "drei".to_string()]);
/// ```
#[must_use]
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let w = display_width(word);
        if current.is_empty() {
            current.push_str(word);
            current_width = w;
        } else if current_width + 1 + w <= width {
            current.push(' ');
            current.push_str(word);
            current_width += 1 + w;
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
            current_width = w;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Frames `text` in a cowsay-style speech bubble (plain, uncoloured text).
///
/// `width` is the maximum text width *inside* the bubble.
#[must_use]
pub fn speech_bubble(text: &str, width: usize) -> Vec<String> {
    let lines = wrap(text, width);
    let inner = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);

    let mut out = Vec::with_capacity(lines.len() + 2);
    out.push(format!(" {}", "_".repeat(inner + 2)));

    let n = lines.len();
    for (i, line) in lines.iter().enumerate() {
        let (left, right) = match (n, i) {
            (1, _) => ('<', '>'),
            (_, 0) => ('/', '\\'),
            (_, last) if last == n - 1 => ('\\', '/'),
            _ => ('|', '|'),
        };
        let pad = " ".repeat(inner - display_width(line));
        out.push(format!("{left} {line}{pad} {right}"));
    }

    out.push(format!(" {}", "-".repeat(inner + 2)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_respects_width() {
        let lines = wrap(&"wort ".repeat(20), 12);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| display_width(l) <= 12));
    }

    #[test]
    fn wrap_never_empty() {
        assert_eq!(wrap("", 10), vec![String::new()]);
    }

    #[test]
    fn bubble_frames_text() {
        let b = speech_bubble("Hallo", 40);
        assert!(b[0].contains('_'));
        assert!(b.iter().any(|l| l.contains("Hallo")));
        assert!(b.last().unwrap().contains('-'));
    }

    #[test]
    fn visible_width_ignores_ansi() {
        assert_eq!(visible_width("\u{1b}[33mAB\u{1b}[0m"), 2);
        assert_eq!(visible_width("AB"), 2);
        assert_eq!(visible_width(""), 0);
        // coloured duck: visible width = 5 ("<( o)")
        assert_eq!(visible_width("\u{1b}[33m<( o)\u{1b}[0m"), 5);
    }

    #[test]
    fn umlauts_pad_by_display_width() {
        let ascii = speech_bubble("ABCD", 40);
        let umlaut = speech_bubble("Üäöß", 40);
        let aw: Vec<usize> = ascii.iter().map(|l| display_width(l)).collect();
        let uw: Vec<usize> = umlaut.iter().map(|l| display_width(l)).collect();
        assert_eq!(aw, uw);
    }
}
