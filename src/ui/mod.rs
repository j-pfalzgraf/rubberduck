//! Terminal interface of rubberduck: theme, animations and the duck.
//!
//! The [`Ui`] is the facade that the rest of the application talks to. It encapsulates
//! colour and TTY resolution, playing back animations and gracefully
//! degrading to static or plain-text output (`--quiet`, no terminal).

pub mod animate;
pub mod bar;
pub mod duck;
pub mod gradient;
pub mod scene;
pub mod spinner;
pub mod surface;
pub mod text;
pub mod theme;

use crate::i18n::{Lang, Tr};
use crate::ui::animate::{Animation, Player, Repeat};
use crate::ui::gradient::Gradient;
use crate::ui::scene::SpeechScene;
use crate::ui::spinner::{SpinnerStyle, Thinking};
use crate::ui::surface::{Surface, TermSurface};
use crate::ui::theme::{ColorChoice, Styler, Theme};
use std::io::{self, IsTerminal};

pub use duck::Mood;

/// ANSI sequence that makes the cursor visible again (`CSI ?25h`).
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";

/// Installs safeguards that restore the cursor if the
/// program is terminated mid-animation by **Ctrl-C** or a **panic**
/// (both skip `Drop`, so RAII guards are not enough).
///
/// Call once at startup. No-op when the respective output is not a terminal
/// (in that case nobody hides the cursor anyway).
pub fn install_terminal_guards() {
    use std::io::Write;

    if std::io::stderr().is_terminal() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let mut err = std::io::stderr();
            let _ = err.write_all(SHOW_CURSOR);
            let _ = err.flush();
            previous(info);
        }));
    }

    if std::io::stdout().is_terminal() {
        let _ = ctrlc::set_handler(|| {
            let mut out = std::io::stdout();
            let _ = out.write_all(SHOW_CURSOR);
            let _ = out.flush();
            std::process::exit(130); // 128 + SIGINT
        });
    }
}

/// Settings used to create a [`Ui`] (derived from the configuration).
#[derive(Debug, Clone)]
pub struct UiSettings {
    /// Colour mode.
    pub color: ColorChoice,
    /// Name of the theme (see [`Theme::NAMES`]).
    pub theme: String,
    /// Whether animations are allowed in general.
    pub animations: bool,
    /// Speed multiplier (1.0 = normal).
    pub speed: f32,
    /// Whether the typewriter effect is active.
    pub typewriter: bool,
    /// `quiet`: no duck/animation, only terse text.
    pub quiet: bool,
    /// User-interface language.
    pub lang: Lang,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            color: ColorChoice::Auto,
            theme: "classic".to_string(),
            animations: true,
            speed: 1.0,
            typewriter: true,
            quiet: false,
            lang: Lang::English,
        }
    }
}

/// The high-level interface of rubberduck.
pub struct Ui {
    styler: Styler,
    settings: UiSettings,
    stdout_tty: bool,
    stdin_tty: bool,
    tr: Tr,
}

impl Ui {
    /// Creates the Ui and resolves colour and TTY questions once.
    #[must_use]
    pub fn new(settings: UiSettings) -> Self {
        let stdout_tty = io::stdout().is_terminal();
        let stdin_tty = io::stdin().is_terminal();
        let styler = Styler::new(Theme::by_name(&settings.theme), settings.color.resolve());
        let tr = Tr::new(settings.lang);
        Self {
            styler,
            settings,
            stdout_tty,
            stdin_tty,
            tr,
        }
    }

    /// The active styler (e.g. for accompanying texts of the application).
    #[must_use]
    pub fn styler(&self) -> &Styler {
        &self.styler
    }

    /// The active translator (bound to the configured language).
    #[must_use]
    pub fn tr(&self) -> Tr {
        self.tr
    }

    /// Whether the output goes to a real terminal.
    #[must_use]
    pub fn is_tty(&self) -> bool {
        self.stdout_tty
    }

    /// Whether an interactive dialog is possible – stdin **and** stdout are
    /// terminals. Only then may selection and yes/no prompts run.
    #[must_use]
    pub fn is_interactive(&self) -> bool {
        self.stdin_tty && self.stdout_tty
    }

    /// Whether in `quiet` mode (no duck).
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.settings.quiet
    }

    /// Whether animations actually run (stdout terminal + enabled + not `quiet`).
    #[must_use]
    pub fn animating(&self) -> bool {
        self.stdout_tty && self.settings.animations && !self.settings.quiet
    }

    /// Plays an arbitrary animation with the current settings (demo/stats use this).
    pub fn play(&mut self, anim: &dyn Animation) -> io::Result<()> {
        let mut surface = TermSurface::stdout();
        Player::new(&mut surface, self.animating(), self.settings.speed).play(anim)
    }

    /// Current terminal width in columns.
    #[must_use]
    pub fn width(&self) -> usize {
        TermSurface::stdout().width() as usize
    }

    /// Prints a one-line gradient banner (a plain accent title in `quiet` mode,
    /// and plain text when colour is off).
    pub fn gradient_banner(&self, text: &str, gradient: &Gradient) {
        if self.settings.quiet {
            println!("{}", self.styler.accent(text));
        } else {
            println!("{}", gradient::paint(text, gradient, self.styler.enabled()));
        }
    }

    /// Prints a colour preview of each built-in theme (just the names in `quiet`
    /// mode, since the duck is suppressed there).
    pub fn theme_previews(&self) {
        for name in Theme::NAMES {
            if self.settings.quiet {
                println!("  {name}");
            } else {
                let styler = Styler::new(Theme::by_name(name), self.styler.enabled());
                println!("  {}", styler.duck(&format!("{name:<9} <( o)___")));
            }
        }
    }

    /// Prints a painted swatch of every named gradient (names only in `quiet`
    /// mode). Drives the demo's gradient showcase.
    pub fn gradient_previews(&self) {
        for (name, gradient) in Gradient::showcase() {
            if self.settings.quiet {
                println!("  {name}");
            } else {
                let swatch = gradient::paint(&"█".repeat(18), &gradient, self.styler.enabled());
                println!("  {name:<9} {swatch}");
            }
        }
    }

    /// Inner speech-bubble width matching the terminal width.
    fn bubble_width(&self, surface: &impl Surface) -> usize {
        (surface.width() as usize).saturating_sub(8).clamp(16, 64)
    }

    /// The duck speaks `text` (typewriter effect + duck) with mood `mood`.
    ///
    /// In `quiet` mode only a terse line of text is printed instead.
    pub fn duck_says(&mut self, text: &str, mood: Mood) -> io::Result<()> {
        if self.settings.quiet {
            println!("\n{} {}", self.styler.accent("?"), self.styler.text(text));
            return Ok(());
        }
        let mut surface = TermSurface::stdout();
        let width = self.bubble_width(&surface);
        let scene = SpeechScene::new(text, width, mood, self.styler, self.typewriter_active());
        Player::new(&mut surface, self.animating(), self.settings.speed).play(&scene)
    }

    /// Lets the duck swim into the frame and settle with a blink, as one fluid
    /// entrance (only when animated; otherwise a no-op).
    pub fn swim_in(&mut self, mood: Mood) -> io::Result<()> {
        if !self.animating() {
            return Ok(());
        }
        let mut surface = TermSurface::stdout();
        let width = surface.width() as usize;
        let anim = duck::entrance(mood, self.styler, width);
        Player::new(&mut surface, true, self.settings.speed).play(&anim)
    }

    /// Plays a short quack animation.
    pub fn quack(&mut self, mood: Mood) -> io::Result<()> {
        if self.settings.quiet {
            return Ok(());
        }
        let mut surface = TermSurface::stdout();
        let clip = duck::quack_clip(mood, self.styler, self.tr.quack_word());
        Player::new(&mut surface, self.animating(), self.settings.speed).play(&clip)
    }

    /// Plays the celebration animation for the aha moment.
    pub fn celebrate(&mut self) -> io::Result<()> {
        if self.settings.quiet {
            println!("\n{}", self.styler.success(self.tr.celebrate_quiet()));
            return Ok(());
        }
        let mut surface = TermSurface::stdout();
        let width = surface.width() as usize;
        let gradient = Gradient::rainbow();
        let clip = duck::celebrate_clip(self.styler, self.tr.eureka(), width, &gradient);
        Player::new(&mut surface, self.animating(), self.settings.speed).play(&clip)
    }

    /// Shows a thinking spinner for `cycles` frames.
    pub fn thinking(&mut self, label: &str, cycles: usize) -> io::Result<()> {
        if !self.animating() {
            return Ok(());
        }
        let mut surface = TermSurface::stdout();
        let spinner = Thinking::new(label, self.styler, cycles);
        Player::new(&mut surface, true, self.settings.speed).play(&spinner)
    }

    /// Plays a labelled spinner in `style` for two full cycles (demo showcase).
    ///
    /// The base [`Thinking`] cycle is wrapped in [`Repeat`], so the cycle is
    /// defined once and the repeat count stays a separate concern.
    pub fn spinner_showcase(&mut self, style: SpinnerStyle) -> io::Result<()> {
        let mut surface = TermSurface::stdout();
        let base = Thinking::styled(style.name(), self.styler, style.cycle_len(), style);
        let anim = Repeat::new(Box::new(base), 2);
        Player::new(&mut surface, self.animating(), self.settings.speed).play(&anim)
    }

    fn typewriter_active(&self) -> bool {
        self.settings.typewriter && self.animating()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_sane() {
        let s = UiSettings::default();
        assert!(s.animations);
        assert!(s.typewriter);
        assert!((s.speed - 1.0).abs() < f32::EPSILON);
    }
}
