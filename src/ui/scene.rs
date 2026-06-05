//! Zusammengesetzte Szene: Sprechblase (Tippeffekt) über der animierten Ente.
//!
//! [`SpeechScene`] ist selbst eine [`Animation`]: Bild `i` zeigt die ersten `i`
//! Zeichen des Texts, während die Ente darunter gelegentlich blinzelt. Ohne
//! Tippeffekt besteht die Animation aus genau einem Bild (sofort vollständig).

use crate::ui::animate::{Animation, Frame};
use crate::ui::duck::{self, Mood};
use crate::ui::text;
use crate::ui::theme::Styler;
use std::time::Duration;

/// Anzahl zusätzlicher „Halte“-Bilder nach dem Austippen (für ein Blinzeln).
const HOLD_FRAMES: usize = 6;

/// Eine Frage-Szene: die Ente „spricht“ den Text Zeichen für Zeichen aus.
pub struct SpeechScene {
    text: Vec<char>,
    bubble_width: usize,
    mood: Mood,
    styler: Styler,
    typewriter: bool,
}

impl SpeechScene {
    /// Neue Szene. `typewriter=false` zeigt sofort den ganzen Text (ein Bild).
    #[must_use]
    pub fn new(
        text: &str,
        bubble_width: usize,
        mood: Mood,
        styler: Styler,
        typewriter: bool,
    ) -> Self {
        Self {
            text: text.chars().collect(),
            bubble_width: bubble_width.max(8),
            mood,
            styler,
            typewriter,
        }
    }

    fn revealed(&self, i: usize) -> String {
        let shown = if self.typewriter {
            i.min(self.text.len())
        } else {
            self.text.len()
        };
        self.text[..shown].iter().collect()
    }

    fn compose(&self, revealed: &str, eye: char) -> Frame {
        let mut lines = Vec::new();
        for line in text::speech_bubble(revealed, self.bubble_width) {
            lines.push(self.styler.bubble(&line));
        }
        lines.push(self.styler.dim("    \\"));
        lines.push(self.styler.dim("     \\"));
        for line in duck::posed(self.mood, eye) {
            lines.push(self.styler.duck(&line));
        }
        Frame::new(lines)
    }
}

impl Animation for SpeechScene {
    fn frame_count(&self) -> usize {
        if self.typewriter {
            self.text.len() + HOLD_FRAMES
        } else {
            1
        }
    }

    fn frame(&self, i: usize) -> Frame {
        let revealed = self.revealed(i);
        // Während des Tippens Auge offen; in der Halte-Phase gelegentlich blinzeln.
        let done_at = self.text.len();
        let eye = if self.typewriter && i >= done_at && (i - done_at) % 2 == 1 {
            duck::blink_eye(self.mood)
        } else {
            duck::eye_for(self.mood)
        };
        self.compose(&revealed, eye)
    }

    fn delay(&self, i: usize) -> Duration {
        match self.text.get(i) {
            // Natürlicher Rhythmus: nach Satzzeichen kurz innehalten.
            Some('.' | '!' | '?') => Duration::from_millis(180),
            Some(',' | ';' | ':') => Duration::from_millis(110),
            Some(_) => Duration::from_millis(22),
            None => Duration::from_millis(170),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;

    fn styler() -> Styler {
        Styler::new(Theme::CLASSIC, false)
    }

    #[test]
    fn non_typewriter_is_single_full_frame() {
        let scene = SpeechScene::new("Hallo Ente", 40, Mood::Idle, styler(), false);
        assert_eq!(scene.frame_count(), 1);
        let f = scene.frame(0);
        assert!(f.lines.iter().any(|l| l.contains("Hallo Ente")));
        assert!(f.lines.iter().any(|l| l.contains("<( o)")));
    }

    #[test]
    fn typewriter_reveals_progressively() {
        let scene = SpeechScene::new("abc", 40, Mood::Idle, styler(), true);
        assert_eq!(scene.frame_count(), 3 + HOLD_FRAMES);
        let early = scene.frame(1);
        let full = scene.frame(scene.frame_count() - 1);
        let early_text: String = early.lines.join("\n");
        let full_text: String = full.lines.join("\n");
        assert!(early_text.matches('a').count() >= 1);
        assert!(full_text.contains("abc"));
    }
}
