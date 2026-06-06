//! Frame-by-frame animations and playing them on a [`Surface`].
//!
//! An [`Animation`] supplies any number of [`Frame`]s; the [`Player`] redraws
//! them in place (cursor up, clear, rewrite). If the player is disabled (no TTY
//! or `--no-anim`), only the *last* frame is drawn once – so every animation
//! degrades cleanly to static output.

use crate::ui::surface::Surface;
use std::io;
use std::time::Duration;

/// A single frame: several (already colourised) text lines.
#[derive(Debug, Clone, Default)]
pub struct Frame {
    /// The lines of the frame (without line breaks).
    pub lines: Vec<String>,
}

impl Frame {
    /// Frame from lines.
    #[must_use]
    pub fn new(lines: Vec<String>) -> Self {
        Self { lines }
    }

    /// Height in lines.
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

/// Something that can be played back as a finite sequence of frames.
pub trait Animation {
    /// Number of frames.
    fn frame_count(&self) -> usize;

    /// Renders frame `i` (0-based, `i < frame_count()`).
    fn frame(&self, i: usize) -> Frame;

    /// Delay *after* frame `i` (default: 90 ms).
    fn delay(&self, _i: usize) -> Duration {
        Duration::from_millis(90)
    }

    /// Whether the animation has no frame at all.
    fn is_empty(&self) -> bool {
        self.frame_count() == 0
    }
}

/// An animation made of pre-rendered frames with a fixed frame delay.
pub struct Clip {
    frames: Vec<Frame>,
    delay: Duration,
}

impl Clip {
    /// Clip from frames and a uniform delay.
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

/// Easing curves for movements; map `t ∈ [0,1]` onto `[0,1]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    /// Uniform.
    Linear,
    /// Smooth acceleration and deceleration.
    EaseInOut,
    /// Springy impact at the end.
    Bounce,
}

impl Easing {
    /// Applies the curve to `t` (clamped outside `[0,1]`).
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

/// Plays [`Animation`]s on a [`Surface`].
pub struct Player<'a, S: Surface> {
    surface: &'a mut S,
    enabled: bool,
    speed: f32,
}

impl<'a, S: Surface> Player<'a, S> {
    /// New player. With `enabled=false` only the final frame is drawn; `speed`
    /// scales the delays (2.0 = twice as fast). `speed <= 0` is treated as 1.0.
    pub fn new(surface: &'a mut S, enabled: bool, speed: f32) -> Self {
        Self {
            surface,
            enabled,
            speed: if speed <= 0.0 { 1.0 } else { speed },
        }
    }

    /// Plays `anim` and leaves the final frame on the screen.
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
        // ALWAYS restore the cursor – even if drawing failed, otherwise it would
        // remain invisible in the user's terminal.
        let _ = self.surface.show_cursor();
        let _ = self.surface.flush();
        result
    }

    /// Draws the frames in sequence (cursor handling is done by [`Player::play`]).
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

    /// Number of terminal rows that `frame` occupies – including soft-wrapping of
    /// wide lines. Without this correction the in-place redraw miscalculates on
    /// narrow terminals.
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
        // BufferSurface ignores clear -> all frames accumulate in sequence.
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
