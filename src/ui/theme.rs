//! Farbschemata und das Einfärben von Ausgaben.
//!
//! [`ColorChoice`] entscheidet, *ob* gefärbt wird (mit `NO_COLOR`-Unterstützung),
//! [`Theme`] legt die Palette fest und [`Styler`] wendet sie an. Ist Farbe
//! deaktiviert, geben alle Styler-Methoden den Text unverändert zurück – die
//! Ausgabe bleibt damit pipe- und logfreundlich.

use crossterm::style::Color;
use std::io::IsTerminal;

/// Steuert, wann farbige Ausgaben erzeugt werden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    /// Farbe nur, wenn stdout ein Terminal ist und `NO_COLOR` nicht gesetzt ist.
    Auto,
    /// Immer farbig (auch in Pipes).
    Always,
    /// Niemals farbig.
    Never,
}

impl ColorChoice {
    /// Löst die Wahl gegen das aktuelle stdout und die `NO_COLOR`-Konvention auf.
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

/// Eine Farbpalette für die Enten-Ausgabe.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Name des Themes (z. B. `"classic"`).
    pub name: &'static str,
    /// Farbe der Ente selbst.
    pub duck: Color,
    /// Akzentfarbe für Hervorhebungen.
    pub accent: Color,
    /// Farbe des Sprechblasen-Rahmens und -Texts.
    pub bubble: Color,
    /// Farbe für normalen Fließtext.
    pub text: Color,
    /// Gedämpfte Farbe für Nebeninformationen.
    pub dim: Color,
    /// Erfolgs-/Feierfarbe.
    pub success: Color,
    /// Farbe der Wasserlinie.
    pub water: Color,
}

impl Theme {
    /// Namen aller eingebauten Themes.
    pub const NAMES: &'static [&'static str] = &["classic", "midnight", "mono"];

    /// Warmes Standard-Theme mit gelber Ente.
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

    /// Dunkles Theme mit sanften RGB-Tönen.
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

    /// Monochromes Theme (ein Farbton) für nüchterne Terminals.
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

    /// Theme per Name; ein unbekannter Name liefert [`Theme::CLASSIC`].
    #[must_use]
    pub fn by_name(name: &str) -> Theme {
        match name {
            "midnight" => Self::MIDNIGHT,
            "mono" => Self::MONO,
            _ => Self::CLASSIC,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::CLASSIC
    }
}

/// Färbt Strings gemäß einem [`Theme`] – oder gibt sie unverändert zurück, wenn
/// Farbe deaktiviert ist.
#[derive(Debug, Clone, Copy)]
pub struct Styler {
    theme: Theme,
    enabled: bool,
}

impl Styler {
    /// Neuer Styler mit Palette `theme`; `enabled=false` schaltet Farbe ganz aus.
    #[must_use]
    pub fn new(theme: Theme, enabled: bool) -> Self {
        Self { theme, enabled }
    }

    /// Die zugrunde liegende Palette.
    #[must_use]
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Ob dieser Styler tatsächlich färbt.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Kernroutine: färbt `s` mit Farbe `c` (oder gibt `s` unverändert zurück).
    #[must_use]
    pub fn paint(&self, s: &str, c: Color) -> String {
        if !self.enabled {
            return s.to_string();
        }
        use crossterm::style::{ResetColor, SetForegroundColor};
        format!("{}{}{}", SetForegroundColor(c), s, ResetColor)
    }

    /// Färbt `s` in der Entenfarbe.
    #[must_use]
    pub fn duck(&self, s: &str) -> String {
        self.paint(s, self.theme.duck)
    }
    /// Färbt `s` in der Akzentfarbe.
    #[must_use]
    pub fn accent(&self, s: &str) -> String {
        self.paint(s, self.theme.accent)
    }
    /// Färbt `s` in der Sprechblasenfarbe.
    #[must_use]
    pub fn bubble(&self, s: &str) -> String {
        self.paint(s, self.theme.bubble)
    }
    /// Färbt `s` in der Textfarbe.
    #[must_use]
    pub fn text(&self, s: &str) -> String {
        self.paint(s, self.theme.text)
    }
    /// Färbt `s` gedämpft.
    #[must_use]
    pub fn dim(&self, s: &str) -> String {
        self.paint(s, self.theme.dim)
    }
    /// Färbt `s` in der Erfolgsfarbe.
    #[must_use]
    pub fn success(&self, s: &str) -> String {
        self.paint(s, self.theme.success)
    }
    /// Färbt `s` in der Wasserfarbe.
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
        assert!(painted.len() > "Quak".len(), "sollte ANSI-Codes enthalten");
    }

    #[test]
    fn color_choice_never_is_false() {
        assert!(!ColorChoice::Never.resolve());
        assert!(ColorChoice::Always.resolve());
    }

    #[test]
    fn unknown_theme_falls_back_to_classic() {
        assert_eq!(Theme::by_name("gibtsnicht").name, "classic");
        assert_eq!(Theme::by_name("midnight").name, "midnight");
    }
}
