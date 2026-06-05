//! ASCII-Ente samt cowsay-artiger Sprechblase.

use crossterm::style::Stylize;
use std::io::IsTerminal;

/// Maximale Textbreite innerhalb der Sprechblase.
const BUBBLE_WIDTH: usize = 50;

/// Die Ente selbst (inkl. „Schwanz“, der zur Sprechblase zeigt).
const DUCK: &str = r#"      \
       \
        __
      <( o)___
       (___/   quack!
"#;

/// Baut die komplette Ausgabe (Sprechblase + Ente) als reinen Text.
pub fn duck_says(message: &str) -> String {
    format!("{}{DUCK}", speech_bubble(message))
}

/// Gibt die Ente aus – farbig, sofern stdout ein Terminal ist.
pub fn print_duck_says(message: &str) {
    let art = duck_says(message);
    if std::io::stdout().is_terminal() {
        println!("{}", art.yellow());
    } else {
        println!("{art}");
    }
}

/// Rahmt `text` in einer cowsay-artigen Sprechblase ein.
pub fn speech_bubble(text: &str) -> String {
    let lines = wrap(text, BUBBLE_WIDTH);
    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

    let mut out = String::new();
    out.push(' ');
    out.push_str(&"_".repeat(width + 2));
    out.push('\n');

    let n = lines.len();
    for (i, line) in lines.iter().enumerate() {
        let (left, right) = match (n, i) {
            (1, _) => ('<', '>'),
            (_, 0) => ('/', '\\'),
            (_, last) if last == n - 1 => ('\\', '/'),
            _ => ('|', '|'),
        };
        let pad = " ".repeat(width - line.chars().count());
        out.push_str(&format!("{left} {line}{pad} {right}\n"));
    }

    out.push(' ');
    out.push_str(&"-".repeat(width + 2));
    out.push('\n');
    out
}

/// Einfacher Wortumbruch an Leerzeichen; sehr lange Wörter dürfen überstehen.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.chars().count() + 1 + word.chars().count() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bubble_has_borders_and_text() {
        let bubble = speech_bubble("Hallo Ente");
        assert!(bubble.contains("Hallo Ente"));
        assert!(bubble.contains('_'));
        assert!(bubble.contains('-'));
        assert!(bubble.contains('<') && bubble.contains('>'));
    }

    #[test]
    fn long_text_wraps_within_width() {
        let lines = wrap(&"wort ".repeat(40), 20);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| l.chars().count() <= 20));
    }

    #[test]
    fn umlauts_pad_by_chars_not_bytes() {
        // Gleich viele Zeichen -> identische Rahmenbreite, obwohl Umlaute
        // mehr Bytes brauchen. Schlägt fehl, falls nach Bytes gepaddet würde.
        let ascii = speech_bubble("ABCD");
        let umlaut = speech_bubble("Üäöß");
        let ascii_widths: Vec<usize> = ascii.lines().map(|l| l.chars().count()).collect();
        let umlaut_widths: Vec<usize> = umlaut.lines().map(|l| l.chars().count()).collect();
        assert_eq!(ascii_widths, umlaut_widths);
    }

    #[test]
    fn duck_contains_art_and_message() {
        let duck = duck_says("Quak");
        assert!(duck.contains("<( o)"));
        assert!(duck.contains("Quak"));
    }
}
