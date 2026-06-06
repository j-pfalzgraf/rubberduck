//! Composite scene: speech bubble (typewriter effect) above the animated duck.
//!
//! [`SpeechScene`] is itself an [`Animation`]: frame `i` shows the first `i`
//! characters of the text, while the duck below blinks occasionally. Without
//! the typewriter effect, the animation consists of exactly one frame (complete immediately).

use crate::ui::animate::{Animation, Frame};
use crate::ui::duck::{self, Mood};
use crate::ui::text;
use crate::ui::theme::Styler;
use std::time::Duration;

/// Number of additional "hold" frames after typing finishes (for a blink).
const HOLD_FRAMES: usize = 6;

/// A question scene: the duck "speaks" the text character by character.
pub struct SpeechScene {
    text: Vec<char>,
    bubble_width: usize,
    mood: Mood,
    styler: Styler,
    typewriter: bool,
}

impl SpeechScene {
    /// New scene. `typewriter=false` shows the whole text immediately (one frame).
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
        // Eye open while typing; blink occasionally during the hold phase.
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
            // Natural rhythm: pause briefly after punctuation marks.
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
        let scene = SpeechScene::new("Hello duck", 40, Mood::Idle, styler(), false);
        assert_eq!(scene.frame_count(), 1);
        let f = scene.frame(0);
        assert!(f.lines.iter().any(|l| l.contains("Hello duck")));
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
