//! Bild-für-Bild-Animationen und ihr Abspielen auf einer [`Surface`].
//!
//! Eine [`Animation`] liefert beliebig viele [`Frame`]s; der [`Player`] zeichnet
//! sie an Ort und Stelle neu (Cursor hoch, löschen, neu schreiben). Ist der
//! Player deaktiviert (kein TTY oder `--no-anim`), wird nur das *letzte* Bild
//! einmal gezeichnet – so degradiert jede Animation sauber zu statischer Ausgabe.

use crate::ui::surface::Surface;
use std::io;
use std::time::Duration;

/// Ein einzelnes Bild: mehrere (bereits eingefärbte) Textzeilen.
#[derive(Debug, Clone, Default)]
pub struct Frame {
    /// Die Zeilen des Bildes (ohne Zeilenumbrüche).
    pub lines: Vec<String>,
}

impl Frame {
    /// Bild aus Zeilen.
    #[must_use]
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }

    /// Höhe in Zeilen.
    #[must_use]
    pub fn height(&self) -> usize {
        self.lines.len()
    }
}

impl From<Vec<String>> for Frame {
    fn from(lines: Vec<String>) -> Self {
        Self { lines }
    }
}

/// Etwas, das als endliche Folge von Bildern abgespielt werden kann.
pub trait Animation {
    /// Anzahl der Bilder.
    fn frame_count(&self) -> usize;

    /// Rendert Bild `i` (0-basiert, `i < frame_count()`).
    fn frame(&self, i: usize) -> Frame;

    /// Verzögerung *nach* Bild `i` (Standard: 90 ms).
    fn delay(&self, _i: usize) -> Duration {
        Duration::from_millis(90)
    }

    /// Ob die Animation kein einziges Bild hat.
    fn is_empty(&self) -> bool {
        self.frame_count() == 0
    }
}

/// Eine Animation aus vorab gerenderten Bildern mit fixer Bildverzögerung.
pub struct Clip {
    frames: Vec<Frame>,
    delay: Duration,
}

impl Clip {
    /// Clip aus Bildern und einheitlicher Verzögerung.
    #[must_use]
    pub fn new(frames: Vec<Frame>, delay: Duration) -> Self {
        Self { frames, delay }
    }
}

impl Animation for Clip {
    fn frame_count(&self) -> usize {
        self.frames.len()
    }
    fn frame(&self, i: usize) -> Frame {
        self.frames[i].clone()
    }
    fn delay(&self, _i: usize) -> Duration {
        self.delay
    }
}

/// Beschleunigungskurven für Bewegungen; bilden `t ∈ [0,1]` auf `[0,1]` ab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    /// Gleichförmig.
    Linear,
    /// Weiches Anfahren und Abbremsen.
    EaseInOut,
    /// Federndes Aufschlagen am Ende.
    Bounce,
}

impl Easing {
    /// Wendet die Kurve auf `t` an (außerhalb `[0,1]` wird geklemmt).
    #[must_use]
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::Bounce => {
                let n = 7.5625;
                let d = 2.75;
                if t < 1.0 / d {
                    n * t * t
                } else if t < 2.0 / d {
                    let t = t - 1.5 / d;
                    n * t * t + 0.75
                } else if t < 2.5 / d {
                    let t = t - 2.25 / d;
                    n * t * t + 0.9375
                } else {
                    let t = t - 2.625 / d;
                    n * t * t + 0.984375
                }
            }
        }
    }
}

/// Spielt [`Animation`]s auf einer [`Surface`] ab.
pub struct Player<'a, S: Surface> {
    surface: &'a mut S,
    enabled: bool,
    speed: f32,
}

impl<'a, S: Surface> Player<'a, S> {
    /// Neuer Player. Bei `enabled=false` wird nur das Endbild gezeichnet; `speed`
    /// skaliert die Verzögerungen (2.0 = doppelt so schnell). `speed <= 0` gilt
    /// als 1.0.
    pub fn new(surface: &'a mut S, enabled: bool, speed: f32) -> Self {
        Self {
            surface,
            enabled,
            speed: if speed <= 0.0 { 1.0 } else { speed },
        }
    }

    /// Spielt `anim` ab und lässt das Endbild auf dem Schirm stehen.
    pub fn play(&mut self, anim: &dyn Animation) -> io::Result<()> {
        let count = anim.frame_count();
        if count == 0 {
            return Ok(());
        }
        if !self.enabled || count == 1 {
            return self.draw_block(&anim.frame(count - 1));
        }

        self.surface.hide_cursor()?;
        let result = self.play_frames(anim, count);
        // Cursor IMMER wiederherstellen – auch wenn das Zeichnen fehlschlug,
        // sonst bliebe er im Terminal der Nutzerin unsichtbar.
        let _ = self.surface.show_cursor();
        let _ = self.surface.flush();
        result
    }

    /// Zeichnet die Bilder der Reihe nach (Cursor-Handling liegt bei [`Player::play`]).
    fn play_frames(&mut self, anim: &dyn Animation, count: usize) -> io::Result<()> {
        let mut prev_rows = 0u16;
        for i in 0..count {
            let frame = anim.frame(i);
            if prev_rows > 0 {
                self.surface.move_up(prev_rows)?;
            }
            self.surface.clear_below()?;
            for line in &frame.lines {
                self.surface.write_line(line)?;
            }
            self.surface.flush()?;
            prev_rows = self.frame_rows(&frame);
            if i + 1 < count {
                std::thread::sleep(self.scaled(anim.delay(i)));
            }
        }
        Ok(())
    }

    /// Zahl der Terminalzeilen, die `frame` belegt – inklusive Soft-Wrapping
    /// breiter Zeilen. Ohne diese Korrektur verrechnet sich der In-Place-Redraw
    /// auf schmalen Terminals.
    fn frame_rows(&self, frame: &Frame) -> u16 {
        let width = self.surface.width().max(1) as usize;
        let rows: usize = frame
            .lines
            .iter()
            .map(|line| crate::ui::text::visible_width(line).saturating_sub(1) / width + 1)
            .sum();
        rows.min(u16::MAX as usize) as u16
    }

    fn draw_block(&mut self, frame: &Frame) -> io::Result<()> {
        for line in &frame.lines {
            self.surface.write_line(line)?;
        }
        self.surface.flush()
    }

    fn scaled(&self, d: Duration) -> Duration {
        d.div_f32(self.speed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::surface::BufferSurface;

    fn clip() -> Clip {
        Clip::new(
            vec![
                Frame::new(vec!["A".into()]),
                Frame::new(vec!["B".into()]),
                Frame::new(vec!["C-final".into()]),
            ],
            Duration::from_millis(0),
        )
    }

    #[test]
    fn easing_endpoints() {
        for e in [Easing::Linear, Easing::EaseInOut, Easing::Bounce] {
            assert!((e.apply(0.0) - 0.0).abs() < 1e-3, "{e:?} f(0)");
            assert!((e.apply(1.0) - 1.0).abs() < 1e-3, "{e:?} f(1)");
        }
    }

    #[test]
    fn disabled_player_draws_only_final_frame() {
        let mut surf = BufferSurface::new(40);
        let mut player = Player::new(&mut surf, false, 1.0);
        player.play(&clip()).unwrap();
        assert_eq!(surf.out, "C-final\n");
    }

    #[test]
    fn enabled_player_draws_every_frame() {
        let mut surf = BufferSurface::new(40);
        let mut player = Player::new(&mut surf, true, 100.0);
        player.play(&clip()).unwrap();
        // BufferSurface ignoriert clear -> alle Frames akkumulieren der Reihe nach.
        assert!(surf.out.contains("A\n"));
        assert!(surf.out.contains("B\n"));
        assert!(surf.out.ends_with("C-final\n"));
    }

    #[test]
    fn empty_animation_is_noop() {
        let mut surf = BufferSurface::new(40);
        let mut player = Player::new(&mut surf, true, 1.0);
        player
            .play(&Clip::new(vec![], Duration::from_millis(0)))
            .unwrap();
        assert!(surf.out.is_empty());
    }
}
