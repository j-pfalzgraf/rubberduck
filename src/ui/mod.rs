//! Terminal-Oberfläche von rubberduck: Theme, Animationen und die Ente.
//!
//! Die [`Ui`] ist die Fassade, mit der die übrige Anwendung spricht. Sie kapselt
//! Farb- und TTY-Auflösung, das Abspielen von Animationen und das saubere
//! Degradieren zu statischer bzw. reiner Textausgabe (`--quiet`, kein Terminal).

pub mod animate;
pub mod duck;
pub mod scene;
pub mod spinner;
pub mod surface;
pub mod text;
pub mod theme;

use crate::ui::animate::Player;
use crate::ui::scene::SpeechScene;
use crate::ui::spinner::Thinking;
use crate::ui::surface::{Surface, TermSurface};
use crate::ui::theme::{ColorChoice, Styler, Theme};
use std::io::{self, IsTerminal};

pub use duck::Mood;

/// ANSI-Sequenz, die den Cursor wieder sichtbar macht (`CSI ?25h`).
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";

/// Installiert Schutzmechanismen, die den Cursor wiederherstellen, falls das
/// Programm mitten in einer Animation durch **Strg-C** oder einen **Panic**
/// beendet wird (beides überspringt `Drop`, daher reichen RAII-Guards nicht).
///
/// Einmal beim Start aufrufen. No-op, wenn die jeweilige Ausgabe kein Terminal
/// ist (dann blendet ohnehin niemand den Cursor aus).
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

/// Einstellungen, mit denen eine [`Ui`] erzeugt wird (von der Konfiguration abgeleitet).
#[derive(Debug, Clone)]
pub struct UiSettings {
    /// Farbmodus.
    pub color: ColorChoice,
    /// Name des Themes (siehe [`Theme::NAMES`]).
    pub theme: String,
    /// Ob Animationen grundsätzlich erlaubt sind.
    pub animations: bool,
    /// Geschwindigkeits-Multiplikator (1.0 = normal).
    pub speed: f32,
    /// Ob der Tippeffekt aktiv ist.
    pub typewriter: bool,
    /// `quiet`: keine Ente/Animation, nur knapper Text.
    pub quiet: bool,
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
        }
    }
}

/// Die Hochsprach-Oberfläche von rubberduck.
pub struct Ui {
    styler: Styler,
    settings: UiSettings,
    stdout_tty: bool,
    stdin_tty: bool,
}

impl Ui {
    /// Erzeugt die Ui und löst Farb- und TTY-Fragen einmalig auf.
    #[must_use]
    pub fn new(settings: UiSettings) -> Self {
        let stdout_tty = io::stdout().is_terminal();
        let stdin_tty = io::stdin().is_terminal();
        let styler = Styler::new(Theme::by_name(&settings.theme), settings.color.resolve());
        Self {
            styler,
            settings,
            stdout_tty,
            stdin_tty,
        }
    }

    /// Der aktive Styler (z. B. für Begleittexte der Anwendung).
    #[must_use]
    pub fn styler(&self) -> &Styler {
        &self.styler
    }

    /// Ob die Ausgabe an ein echtes Terminal geht.
    #[must_use]
    pub fn is_tty(&self) -> bool {
        self.stdout_tty
    }

    /// Ob ein interaktiver Dialog möglich ist – stdin **und** stdout sind
    /// Terminals. Nur dann dürfen Auswahl- und Ja/Nein-Prompts laufen.
    #[must_use]
    pub fn is_interactive(&self) -> bool {
        self.stdin_tty && self.stdout_tty
    }

    /// Ob im `quiet`-Modus (keine Ente).
    #[must_use]
    pub fn is_quiet(&self) -> bool {
        self.settings.quiet
    }

    /// Ob Animationen tatsächlich laufen (stdout-Terminal + aktiviert + nicht `quiet`).
    #[must_use]
    pub fn animating(&self) -> bool {
        self.stdout_tty && self.settings.animations && !self.settings.quiet
    }

    /// Innere Sprechblasenbreite passend zur Terminalbreite.
    fn bubble_width(&self, surface: &impl Surface) -> usize {
        (surface.width() as usize).saturating_sub(8).clamp(16, 64)
    }

    /// Die Ente spricht `text` aus (Tippeffekt + Ente) mit Stimmung `mood`.
    ///
    /// Im `quiet`-Modus wird stattdessen nur eine knappe Textzeile gedruckt.
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

    /// Lässt die Ente ins Bild schwimmen (nur wenn animiert; sonst No-op).
    pub fn swim_in(&mut self, mood: Mood) -> io::Result<()> {
        if !self.animating() {
            return Ok(());
        }
        let mut surface = TermSurface::stdout();
        let width = surface.width() as usize;
        let anim = duck::swim_in(mood, self.styler, width);
        Player::new(&mut surface, true, self.settings.speed).play(&anim)
    }

    /// Spielt eine kurze Quak-Animation.
    pub fn quack(&mut self, mood: Mood) -> io::Result<()> {
        if self.settings.quiet {
            return Ok(());
        }
        let mut surface = TermSurface::stdout();
        let clip = duck::quack_clip(mood, self.styler);
        Player::new(&mut surface, self.animating(), self.settings.speed).play(&clip)
    }

    /// Spielt die Feier-Animation für den Aha-Moment.
    pub fn celebrate(&mut self) -> io::Result<()> {
        if self.settings.quiet {
            println!("\n{}", self.styler.success("✨ Stark – gefunden!"));
            return Ok(());
        }
        let mut surface = TermSurface::stdout();
        let clip = duck::celebrate_clip(self.styler);
        Player::new(&mut surface, self.animating(), self.settings.speed).play(&clip)
    }

    /// Zeigt für `cycles` Bilder einen Denk-Spinner.
    pub fn thinking(&mut self, label: &str, cycles: usize) -> io::Result<()> {
        if !self.animating() {
            return Ok(());
        }
        let mut surface = TermSurface::stdout();
        let spinner = Thinking::new(label, self.styler, cycles);
        Player::new(&mut surface, true, self.settings.speed).play(&spinner)
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
