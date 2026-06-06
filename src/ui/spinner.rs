//! A "The duck is thinking …" spinner as a short, finite animation.
//!
//! [`SpinnerStyle`] picks the glyph set; [`Thinking`] turns it into a labelled
//! [`Animation`]. The default style is Braille, matching the classic CLI look.

use crate::ui::animate::{Animation, Frame};
use crate::ui::theme::Styler;
use std::time::Duration;

/// A spinner glyph set. Each style is a short loop of single-cell symbols
/// (except [`SpinnerStyle::Moon`], whose emoji are two cells wide — still safe,
/// because the spinner renders on its own line).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerStyle {
    /// Rotating Braille dots (the default).
    Braille,
    /// A single dot orbiting the cell.
    Dots,
    /// The classic `| / - \` line spinner.
    Line,
    /// A rotating arc.
    Arc,
    /// A bouncing column of Braille dots.
    Bounce,
    /// Moon phases.
    Moon,
}

impl SpinnerStyle {
    /// Every style, in showcase order.
    pub const ALL: [SpinnerStyle; 6] = [
        SpinnerStyle::Braille,
        SpinnerStyle::Dots,
        SpinnerStyle::Line,
        SpinnerStyle::Arc,
        SpinnerStyle::Bounce,
        SpinnerStyle::Moon,
    ];

    /// The animation frames of this style.
    #[must_use]
    pub fn glyphs(self) -> &'static [&'static str] {
        match self {
            SpinnerStyle::Braille => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            SpinnerStyle::Dots => &["⠈", "⠐", "⠠", "⢀", "⡀", "⠄", "⠂", "⠁"],
            SpinnerStyle::Line => &["|", "/", "-", "\\"],
            SpinnerStyle::Arc => &["◜", "◠", "◝", "◞", "◡", "◟"],
            SpinnerStyle::Bounce => &["⢄", "⢂", "⢁", "⡁", "⡈", "⡐", "⡠"],
            SpinnerStyle::Moon => &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
        }
    }

    /// The technical style name (untranslated, like theme names).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            SpinnerStyle::Braille => "braille",
            SpinnerStyle::Dots => "dots",
            SpinnerStyle::Line => "line",
            SpinnerStyle::Arc => "arc",
            SpinnerStyle::Bounce => "bounce",
            SpinnerStyle::Moon => "moon",
        }
    }

    /// Number of frames in one full cycle.
    #[must_use]
    pub fn cycle_len(self) -> usize {
        self.glyphs().len()
    }
}

/// A labelled spinner that runs for `cycles` frames in a chosen [`SpinnerStyle`].
pub struct Thinking {
    label: String,
    styler: Styler,
    style: SpinnerStyle,
    cycles: usize,
}

impl Thinking {
    /// New spinner in the default Braille style with text `label` and `cycles`
    /// frames.
    #[must_use]
    pub fn new(label: &str, styler: Styler, cycles: usize) -> Self {
        Self::styled(label, styler, cycles, SpinnerStyle::Braille)
    }

    /// New spinner in an explicit [`SpinnerStyle`].
    #[must_use]
    pub fn styled(label: &str, styler: Styler, cycles: usize, style: SpinnerStyle) -> Self {
        Self {
            label: label.to_string(),
            styler,
            style,
            cycles,
        }
    }
}

impl Animation for Thinking {
    fn frame_count(&self) -> usize {
        self.cycles
    }

    fn frame(&self, i: usize) -> Frame {
        let glyphs = self.style.glyphs();
        let glyph = glyphs[i % glyphs.len()];
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

    fn styler() -> Styler {
        Styler::new(Theme::CLASSIC, false)
    }

    #[test]
    fn spinner_cycles_through_glyphs() {
        let t = Thinking::new("denkt", styler(), 12);
        assert_eq!(t.frame_count(), 12);
        assert!(t.frame(0).lines[0].contains("denkt"));
        // First and eleventh frames use the same glyph (10-frame Braille cycle).
        assert_eq!(t.frame(0).lines[0], t.frame(10).lines[0]);
    }

    #[test]
    fn every_style_has_glyphs_and_a_name() {
        for style in SpinnerStyle::ALL {
            assert!(style.cycle_len() >= 4, "{} too short", style.name());
            assert!(!style.name().is_empty());
            assert!(style.glyphs().iter().all(|g| !g.is_empty()));
        }
    }

    #[test]
    fn styled_uses_the_chosen_set() {
        let t = Thinking::styled("x", styler(), 4, SpinnerStyle::Line);
        // The Line style's first glyph is "|".
        assert!(t.frame(0).lines[0].contains('|'));
    }
}
