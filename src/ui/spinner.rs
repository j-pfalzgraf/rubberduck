//! Ein „Die Ente überlegt …“-Spinner als kurze, endliche Animation.

use crate::ui::animate::{Animation, Frame};
use crate::ui::theme::Styler;
use std::time::Duration;

/// Die rotierenden Braille-Symbole des Spinners.
const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Ein Spinner mit Beschriftung, der `cycles` Bilder lang läuft.
pub struct Thinking {
    label: String,
    styler: Styler,
    cycles: usize,
}

impl Thinking {
    /// Neuer Spinner mit Text `label` und Anzahl Bilder `cycles`.
    #[must_use]
    pub fn new(label: &str, styler: Styler, cycles: usize) -> Self {
        Self {
            label: label.to_string(),
            styler,
            cycles,
        }
    }
}

impl Animation for Thinking {
    fn frame_count(&self) -> usize {
        self.cycles
    }

    fn frame(&self, i: usize) -> Frame {
        let glyph = FRAMES[i % FRAMES.len()];
        Frame::new(vec![format!(
            "{} {}",
            self.styler.accent(glyph),
            self.styler.dim(&self.label)
        )])
    }

    fn delay(&self, _i: usize) -> Duration {
        Duration::from_millis(80)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::theme::Theme;

    #[test]
    fn spinner_cycles_through_glyphs() {
        let t = Thinking::new("denkt", Styler::new(Theme::CLASSIC, false), 12);
        assert_eq!(t.frame_count(), 12);
        assert!(t.frame(0).lines[0].contains("denkt"));
        // Erstes und elftes Bild nutzen denselben Glyphen (10er-Zyklus).
        assert_eq!(t.frame(0).lines[0], t.frame(10).lines[0]);
    }
}
