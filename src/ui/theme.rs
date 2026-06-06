//! Color schemes and the coloring of output.
//!
//! [`ColorChoice`] decides *whether* coloring happens (with `NO_COLOR` support),
//! [`Theme`] defines the palette, and [`Styler`] applies it. When color is
//! disabled, all styler methods return the text unchanged – this keeps the
//! output pipe- and log-friendly.

use crossterm::style::Color;
use std::io::IsTerminal;

/// Controls when colored output is produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    /// Color only when stdout is a terminal and `NO_COLOR` is not set.
    Auto,
    /// Always colored (even in pipes).
    Always,
    /// Never colored.
    Never,
}

impl ColorChoice {
    /// Resolves the choice against the current stdout and the `NO_COLOR` convention.
    #[must_use]
    pub fn resolve(self) -> bool {
        match self {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => {
                std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
            }
        }
    }
}

/// A color palette for the duck output.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Name of the theme (e.g. `"classic"`).
    pub name: &'static str,
    /// Color of the duck itself.
    pub duck: Color,
    /// Accent color for highlights.
    pub accent: Color,
    /// Color of the speech bubble's border and text.
    pub bubble: Color,
    /// Color for normal body text.
    pub text: Color,
    /// Dimmed color for secondary information.
    pub dim: Color,
    /// Success/celebration color.
    pub success: Color,
    /// Color of the waterline.
    pub water: Color,
}

/// Shorthand for an RGB colour (keeps the theme table compact).
const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

impl Theme {
    /// Names of all built-in themes.
    pub const NAMES: &'static [&'static str] =
        &["classic", "midnight", "mono", "ocean", "forest", "candy"];

    /// Warm default theme with a yellow duck.
    pub const CLASSIC: Theme = Theme {
        name: "classic",
        duck: Color::Yellow,
        accent: Color::Cyan,
        bubble: Color::White,
        text: Color::White,
        dim: Color::DarkGrey,
        success: Color::Green,
        water: Color::Blue,
    };

    /// Dark theme with soft RGB tones.
    pub const MIDNIGHT: Theme = Theme {
        name: "midnight",
        duck: Color::Rgb {
            r: 255,
            g: 209,
            b: 102,
        },
        accent: Color::Rgb {
            r: 130,
            g: 170,
            b: 255,
        },
        bubble: Color::Grey,
        text: Color::White,
        dim: Color::DarkGrey,
        success: Color::Rgb {
            r: 120,
            g: 220,
            b: 160,
        },
        water: Color::Rgb {
            r: 70,
            g: 110,
            b: 200,
        },
    };

    /// Monochrome theme (single color) for plain terminals.
    pub const MONO: Theme = Theme {
        name: "mono",
        duck: Color::White,
        accent: Color::Grey,
        bubble: Color::White,
        text: Color::White,
        dim: Color::DarkGrey,
        success: Color::White,
        water: Color::Grey,
    };

    /// Cool ocean theme.
    pub const OCEAN: Theme = Theme {
        name: "ocean",
        duck: rgb(38, 166, 222),
        accent: rgb(144, 224, 239),
        bubble: Color::Grey,
        text: Color::White,
        dim: Color::DarkGrey,
        success: rgb(64, 200, 160),
        water: rgb(20, 80, 160),
    };

    /// Leafy forest theme.
    pub const FOREST: Theme = Theme {
        name: "forest",
        duck: rgb(124, 193, 86),
        accent: rgb(198, 222, 120),
        bubble: Color::Grey,
        text: Color::White,
        dim: Color::DarkGrey,
        success: rgb(120, 210, 140),
        water: rgb(60, 120, 90),
    };

    /// Sweet candy theme.
    pub const CANDY: Theme = Theme {
        name: "candy",
        duck: rgb(255, 133, 206),
        accent: rgb(255, 214, 128),
        bubble: Color::Grey,
        text: Color::White,
        dim: Color::DarkGrey,
        success: rgb(170, 230, 170),
        water: rgb(160, 120, 220),
    };

    /// Theme by name; an unknown name returns [`Theme::CLASSIC`].
    #[must_use]
    pub fn by_name(name: &str) -> Theme {
        match name {
            "midnight" => Self::MIDNIGHT,
            "mono" => Self::MONO,
            "ocean" => Self::OCEAN,
            "forest" => Self::FOREST,
            "candy" => Self::CANDY,
            _ => Self::CLASSIC,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::CLASSIC
    }
}

/// Colors strings according to a [`Theme`] – or returns them unchanged when
/// color is disabled.
#[derive(Debug, Clone, Copy)]
pub struct Styler {
    theme: Theme,
    enabled: bool,
}

impl Styler {
    /// New styler with palette `theme`; `enabled=false` turns color off entirely.
    #[must_use]
    pub fn new(theme: Theme, enabled: bool) -> Self {
        Self { theme, enabled }
    }

    /// The underlying palette.
    #[must_use]
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Whether this styler actually colors.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Core routine: colors `s` with color `c` (or returns `s` unchanged).
    #[must_use]
    pub fn paint(&self, s: &str, c: Color) -> String {
        if !self.enabled {
            return s.to_string();
        }
        use crossterm::style::{ResetColor, SetForegroundColor};
        format!("{}{}{}", SetForegroundColor(c), s, ResetColor)
    }

    /// Colors `s` in the duck color.
    #[must_use]
    pub fn duck(&self, s: &str) -> String {
        self.paint(s, self.theme.duck)
    }
    /// Colors `s` in the accent color.
    #[must_use]
    pub fn accent(&self, s: &str) -> String {
        self.paint(s, self.theme.accent)
    }
    /// Colors `s` in the speech bubble color.
    #[must_use]
    pub fn bubble(&self, s: &str) -> String {
        self.paint(s, self.theme.bubble)
    }
    /// Colors `s` in the text color.
    #[must_use]
    pub fn text(&self, s: &str) -> String {
        self.paint(s, self.theme.text)
    }
    /// Colors `s` dimmed.
    #[must_use]
    pub fn dim(&self, s: &str) -> String {
        self.paint(s, self.theme.dim)
    }
    /// Colors `s` in the success color.
    #[must_use]
    pub fn success(&self, s: &str) -> String {
        self.paint(s, self.theme.success)
    }
    /// Colors `s` in the water color.
    #[must_use]
    pub fn water(&self, s: &str) -> String {
        self.paint(s, self.theme.water)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_styler_is_identity() {
        let s = Styler::new(Theme::CLASSIC, false);
        assert_eq!(s.duck("Quak"), "Quak");
        assert_eq!(s.accent("x"), "x");
    }

    #[test]
    fn enabled_styler_wraps_with_ansi() {
        let s = Styler::new(Theme::CLASSIC, true);
        let painted = s.duck("Quak");
        assert!(painted.contains("Quak"));
        assert!(painted.len() > "Quak".len(), "should contain ANSI codes");
    }

    #[test]
    fn color_choice_never_is_false() {
        assert!(!ColorChoice::Never.resolve());
        assert!(ColorChoice::Always.resolve());
    }

    #[test]
    fn named_themes_resolve_and_unknown_falls_back() {
        for name in Theme::NAMES {
            assert_eq!(
                Theme::by_name(name).name,
                *name,
                "theme {name} should resolve"
            );
        }
        assert_eq!(Theme::by_name("nope").name, "classic");
    }
}
