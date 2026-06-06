//! The ASCII duck: moods, poses and their animations.
//!
//! [`duck_art`] is the DRY core – a single pose template with an interchangeable
//! eye. Everything else (moods, blinking, swimming, quacking, celebrating) builds
//! on top of it.

use crate::ui::animate::{Animation, Clip, Easing, Frame, Sequence};
use crate::ui::gradient::{self, Gradient};
use crate::ui::theme::Styler;
use std::time::Duration;

/// Mood or pose of the duck.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    /// Neutral, calm.
    Idle,
    /// Thoughtful (thought dots).
    Thinking,
    /// Attentively listening.
    Listening,
    /// Cheerful.
    Happy,
    /// Curious (question mark).
    Curious,
    /// Surprised (wide eyes).
    Surprised,
    /// Celebrating.
    Celebrating,
    /// Sleeping.
    Sleeping,
}

/// The open eye for each mood.
#[must_use]
pub fn eye_for(mood: Mood) -> char {
    match mood {
        Mood::Idle | Mood::Listening | Mood::Thinking | Mood::Curious => 'o',
        Mood::Happy | Mood::Celebrating => '^',
        Mood::Surprised => 'O',
        Mood::Sleeping => '-',
    }
}

/// The "blinked" eye for each mood.
#[must_use]
pub fn blink_eye(mood: Mood) -> char {
    match mood {
        Mood::Surprised => 'o',
        _ => '-',
    }
}

/// The 3-line duck pose with the given eye (DRY core of all poses).
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

/// Adds mood-dependent decorations to a pose.
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

/// Duck pose including decorations for the mood (open eye).
#[must_use]
pub fn duck_for(mood: Mood) -> Vec<String> {
    decorate(duck_art(eye_for(mood)), mood)
}

/// Like [`duck_for`], but with a freely chosen eye (e.g. for blinking).
#[must_use]
pub fn posed(mood: Mood, eye: char) -> Vec<String> {
    decorate(duck_art(eye), mood)
}

/// Colours multiple lines in the duck colour (DRY helper).
fn duck_lines(lines: &[String], styler: Styler) -> Vec<String> {
    lines.iter().map(|l| styler.duck(l)).collect()
}

/// Colours a pose entirely in the duck colour and turns it into a [`Frame`].
fn duck_frame(lines: Vec<String>, styler: Styler) -> Frame {
    Frame::new(duck_lines(&lines, styler))
}

/// Idle animation: the duck blinks occasionally.
#[must_use]
pub fn idle_clip(mood: Mood, styler: Styler) -> Clip {
    let open = duck_frame(duck_art(eye_for(mood)), styler);
    let blink = duck_frame(duck_art(blink_eye(mood)), styler);
    Clip::new(
        vec![open.clone(), open.clone(), open.clone(), blink, open],
        Duration::from_millis(150),
    )
}

/// Quack animation: a localized "quack" word flashes next to the duck.
#[must_use]
pub fn quack_clip(mood: Mood, styler: Styler, word: &str) -> Clip {
    let quiet = duck_frame(duck_art(eye_for(mood)), styler);
    let mut loud_lines = duck_lines(&duck_art(eye_for(mood)), styler);
    loud_lines.push(styler.accent(&format!("   {word} 🦆")));
    let loud = Frame::new(loud_lines);
    Clip::new(
        vec![quiet.clone(), loud.clone(), quiet, loud],
        Duration::from_millis(170),
    )
}

/// Celebration animation for the aha moment: confetti, a gradient banner and a
/// cheering duck.
#[must_use]
pub fn celebrate_clip(styler: Styler, banner: &str, width: usize, gradient: &Gradient) -> Clip {
    let cols = width.clamp(16, 48);
    let frame_for = |phase: usize| {
        let mut lines = vec![
            confetti_row(cols, phase, gradient, styler.enabled()),
            gradient::paint(&format!("✦  {banner}  ✦"), gradient, styler.enabled()),
        ];
        lines.extend(duck_lines(&duck_for(Mood::Celebrating), styler));
        lines.push(styler.accent(r"   \o/  \o/  \o/"));
        lines.push(confetti_row(cols, phase + 2, gradient, styler.enabled()));
        Frame::new(lines)
    };
    Clip::new((0..6).map(frame_for).collect(), Duration::from_millis(150))
}

/// Builds one shifting row of confetti, coloured along the gradient.
fn confetti_row(cols: usize, phase: usize, gradient: &Gradient, enabled: bool) -> String {
    const GLYPHS: [char; 5] = ['·', '✦', '✧', '✶', '*'];
    let mut plain = String::with_capacity(cols);
    for col in 0..cols {
        if (col + phase).is_multiple_of(3) {
            plain.push(GLYPHS[(col + phase) % GLYPHS.len()]);
        } else {
            plain.push(' ');
        }
    }
    gradient::paint(&plain, gradient, enabled)
}

/// The duck swims into frame from the right – with an undulating water line.
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

/// Builds the swim animation (duck `mood`, terminal width `width`).
#[must_use]
pub fn swim_in(mood: Mood, styler: Styler, width: usize) -> impl Animation {
    SwimIn {
        mood,
        styler,
        steps: 14,
        max_pad: width.min(28),
    }
}

/// A fluid entrance: the duck swims in from the right and then **settles** with
/// a blink, played as one continuous [`Sequence`] (no seam between the two).
#[must_use]
pub fn entrance(mood: Mood, styler: Styler, width: usize) -> Sequence {
    Sequence::new(vec![
        Box::new(swim_in(mood, styler, width)),
        Box::new(idle_clip(mood, styler)),
    ])
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
        // First frame indented, last flush (no leading space in the duck head).
        assert!(first.lines[0].starts_with(' '));
        assert!(last.lines[1].starts_with("<("));
    }

    #[test]
    fn entrance_swims_then_settles() {
        let anim = entrance(Mood::Idle, styler(), 40);
        // 14 swim steps + 5 idle frames played as one sequence.
        assert_eq!(anim.frame_count(), 14 + 5);
        // Starts mid-swim (indented), ends on a settled duck with no water line.
        assert!(anim.frame(0).lines[0].starts_with(' '));
        let last = anim.frame(anim.frame_count() - 1);
        assert!(last.lines.iter().all(|l| !l.contains('~')));
        assert!(last.lines[1].starts_with("<("));
    }
}
