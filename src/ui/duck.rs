//! Die ASCII-Ente: Stimmungen, Posen und ihre Animationen.
//!
//! [`duck_art`] ist der DRY-Kern – eine einzige Pose-Vorlage mit austauschbarem
//! Auge. Alles andere (Stimmungen, Blinzeln, Schwimmen, Quaken, Feiern) baut
//! darauf auf.

use crate::ui::animate::{Animation, Clip, Easing, Frame};
use crate::ui::theme::Styler;
use std::time::Duration;

/// Stimmung bzw. Pose der Ente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    /// Neutral, ruhig.
    Idle,
    /// Nachdenklich (Gedankenpunkte).
    Thinking,
    /// Aufmerksam zuhörend.
    Listening,
    /// Fröhlich.
    Happy,
    /// Neugierig (Fragezeichen).
    Curious,
    /// Überrascht (große Augen).
    Surprised,
    /// Feiernd.
    Celebrating,
    /// Schläft.
    Sleeping,
}

/// Das offene Auge je Stimmung.
#[must_use]
pub fn eye_for(mood: Mood) -> char {
    match mood {
        Mood::Idle | Mood::Listening | Mood::Thinking | Mood::Curious => 'o',
        Mood::Happy | Mood::Celebrating => '^',
        Mood::Surprised => 'O',
        Mood::Sleeping => '-',
    }
}

/// Das „geblinzelte“ Auge je Stimmung.
#[must_use]
pub fn blink_eye(mood: Mood) -> char {
    match mood {
        Mood::Surprised => 'o',
        _ => '-',
    }
}

/// Die 3-zeilige Enten-Pose mit gegebenem Auge (DRY-Kern aller Posen).
///
/// ```
/// use rubberduck_cli::ui::duck::duck_art;
/// let lines = duck_art('o');
/// assert_eq!(lines.len(), 3);
/// assert!(lines[1].contains("o"));
/// ```
#[must_use]
pub fn duck_art(eye: char) -> Vec<String> {
    vec![
        "  __".to_string(),
        format!("<( {eye})___"),
        " (___/".to_string(),
    ]
}

/// Versieht eine Pose mit stimmungsabhängigen Verzierungen.
#[must_use]
pub fn decorate(mut art: Vec<String>, mood: Mood) -> Vec<String> {
    match mood {
        Mood::Thinking => art.insert(0, "   . o O".to_string()),
        Mood::Sleeping => art.insert(0, "   z Z".to_string()),
        Mood::Curious => {
            if art.len() > 1 {
                art[1].push_str("  ?");
            }
        }
        Mood::Celebrating => art.insert(0, "  \\ ✨ /".to_string()),
        _ => {}
    }
    art
}

/// Enten-Pose inklusive Verzierungen für die Stimmung (offenes Auge).
#[must_use]
pub fn duck_for(mood: Mood) -> Vec<String> {
    decorate(duck_art(eye_for(mood)), mood)
}

/// Wie [`duck_for`], aber mit frei wählbarem Auge (z. B. zum Blinzeln).
#[must_use]
pub fn posed(mood: Mood, eye: char) -> Vec<String> {
    decorate(duck_art(eye), mood)
}

/// Färbt mehrere Zeilen in der Entenfarbe (DRY-Helfer).
fn duck_lines(lines: &[String], styler: Styler) -> Vec<String> {
    lines.iter().map(|l| styler.duck(l)).collect()
}

/// Färbt eine Pose vollständig in der Entenfarbe und macht daraus ein [`Frame`].
fn duck_frame(lines: Vec<String>, styler: Styler) -> Frame {
    Frame::new(duck_lines(&lines, styler))
}

/// Ruhe-Animation: die Ente blinzelt gelegentlich.
#[must_use]
pub fn idle_clip(mood: Mood, styler: Styler) -> Clip {
    let open = duck_frame(duck_art(eye_for(mood)), styler);
    let blink = duck_frame(duck_art(blink_eye(mood)), styler);
    Clip::new(
        vec![open.clone(), open.clone(), open.clone(), blink, open],
        Duration::from_millis(150),
    )
}

/// Quak-Animation: ein „Quak!“ blitzt neben der Ente auf.
#[must_use]
pub fn quack_clip(mood: Mood, styler: Styler) -> Clip {
    let quiet = duck_frame(duck_art(eye_for(mood)), styler);
    let mut loud_lines = duck_lines(&duck_art(eye_for(mood)), styler);
    loud_lines.push(styler.accent("   Quak! 🦆"));
    let loud = Frame::new(loud_lines);
    Clip::new(
        vec![quiet.clone(), loud.clone(), quiet, loud],
        Duration::from_millis(170),
    )
}

/// Feier-Animation für den Aha-Moment: Funken, Banner und jubelnde Ente.
#[must_use]
pub fn celebrate_clip(styler: Styler) -> Clip {
    let make = |sparkle: &str| {
        let mut lines = vec![styler.success(&format!("   {sparkle}  HEUREKA!  {sparkle}"))];
        lines.extend(duck_lines(&duck_for(Mood::Celebrating), styler));
        lines.push(styler.accent(r"   \o/  \o/  \o/"));
        Frame::new(lines)
    };
    Clip::new(
        vec![
            make("*"),
            make("✦"),
            make("✧"),
            make("✶"),
            make("✦"),
            make("*"),
        ],
        Duration::from_millis(160),
    )
}

/// Die Ente schwimmt von rechts ins Bild – mit wogender Wasserlinie.
struct SwimIn {
    mood: Mood,
    styler: Styler,
    steps: usize,
    max_pad: usize,
}

impl Animation for SwimIn {
    fn frame_count(&self) -> usize {
        self.steps
    }

    fn frame(&self, i: usize) -> Frame {
        let denom = (self.steps - 1).max(1) as f32;
        let eased = Easing::EaseInOut.apply(i as f32 / denom);
        let pad = ((1.0 - eased) * self.max_pad as f32).round() as usize;
        let indent = " ".repeat(pad);

        let indented: Vec<String> = duck_art(eye_for(self.mood))
            .iter()
            .map(|l| format!("{indent}{l}"))
            .collect();
        let mut lines = duck_lines(&indented, self.styler);

        let wave = if i.is_multiple_of(2) {
            "~ ~ ~ ~ ~ ~ ~"
        } else {
            " ~ ~ ~ ~ ~ ~ ~"
        };
        lines.push(self.styler.water(&format!("{indent}{wave}")));
        Frame::new(lines)
    }

    fn delay(&self, _i: usize) -> Duration {
        Duration::from_millis(55)
    }
}

/// Baut die Schwimm-Animation (Ente `mood`, Terminalbreite `width`).
#[must_use]
pub fn swim_in(mood: Mood, styler: Styler, width: usize) -> impl Animation {
    SwimIn {
        mood,
        styler,
        steps: 14,
        max_pad: width.min(28),
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
    fn art_has_three_lines_with_eye() {
        let a = duck_art('@');
        assert_eq!(a.len(), 3);
        assert!(a[1].contains('@'));
    }

    #[test]
    fn moods_decorate() {
        assert!(duck_for(Mood::Curious)[1].contains('?'));
        assert!(duck_for(Mood::Thinking).len() > duck_art('o').len());
    }

    #[test]
    fn idle_blinks_at_least_once() {
        let clip = idle_clip(Mood::Idle, styler());
        assert_eq!(clip.frame_count(), 5);
        let any_blink = (0..clip.frame_count())
            .any(|i| clip.frame(i).lines.iter().any(|l| l.contains("<( -)")));
        assert!(any_blink);
    }

    #[test]
    fn swim_starts_indented_and_ends_flush() {
        let anim = swim_in(Mood::Idle, styler(), 40);
        let first = anim.frame(0);
        let last = anim.frame(anim.frame_count() - 1);
        // Erstes Bild eingerückt, letztes bündig (kein führendes Leerzeichen im Entenkopf).
        assert!(first.lines[0].starts_with(' '));
        assert!(last.lines[1].starts_with("<("));
    }
}
