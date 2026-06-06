//! RGB colour gradients for fancy banners, the celebration and the demo.
//!
//! A [`Gradient`] is a list of RGB stops sampled over `t ∈ [0, 1]`. [`paint`]
//! colours each visible character of a string along the gradient, and degrades
//! to the plain string when colour is disabled (so it stays pipe/log-friendly).

use crossterm::style::{Color, ResetColor, SetForegroundColor};

/// An RGB colour stop.
pub type Stop = (u8, u8, u8);

/// A colour gradient defined by one or more RGB stops.
#[derive(Debug, Clone)]
pub struct Gradient {
    stops: Vec<Stop>,
}

impl Gradient {
    /// Builds a gradient from `stops`. Falls back to white if empty.
    #[must_use]
    pub fn new(stops: Vec<Stop>) -> Self {
        let stops = if stops.is_empty() {
            vec![(255, 255, 255)]
        } else {
            stops
        };
        Self { stops }
    }

    /// A vivid rainbow (red → orange → yellow → green → blue → violet).
    #[must_use]
    pub fn rainbow() -> Self {
        Self::new(vec![
            (255, 79, 79),
            (255, 169, 64),
            (255, 222, 89),
            (120, 220, 120),
            (96, 165, 250),
            (167, 139, 250),
        ])
    }

    /// A warm sunset (matches the classic duck).
    #[must_use]
    pub fn sunset() -> Self {
        Self::new(vec![(255, 209, 102), (255, 138, 76), (236, 90, 130)])
    }

    /// A cool ocean.
    #[must_use]
    pub fn ocean() -> Self {
        Self::new(vec![(70, 110, 200), (96, 200, 220), (120, 230, 180)])
    }

    /// Samples the gradient at `t` (clamped to `[0, 1]`).
    #[must_use]
    pub fn sample(&self, t: f32) -> Stop {
        let t = t.clamp(0.0, 1.0);
        if self.stops.len() == 1 {
            return self.stops[0];
        }
        let segments = self.stops.len() - 1;
        let scaled = t * segments as f32;
        let idx = (scaled.floor() as usize).min(segments - 1);
        let local = scaled - idx as f32;
        lerp(self.stops[idx], self.stops[idx + 1], local)
    }

    /// The crossterm colour at `t`.
    #[must_use]
    pub fn color(&self, t: f32) -> Color {
        let (r, g, b) = self.sample(t);
        Color::Rgb { r, g, b }
    }
}

/// Linear interpolation between two RGB stops.
fn lerp(a: Stop, b: Stop, t: f32) -> Stop {
    let mix = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

/// Colours every character of `s` along `gradient`. Returns plain `s` when
/// `enabled` is false.
#[must_use]
pub fn paint(s: &str, gradient: &Gradient, enabled: bool) -> String {
    if !enabled {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let denom = chars.len().saturating_sub(1).max(1) as f32;
    let mut out = String::with_capacity(s.len() * 8);
    for (i, ch) in chars.iter().enumerate() {
        let t = i as f32 / denom;
        out.push_str(&SetForegroundColor(gradient.color(t)).to_string());
        out.push(*ch);
    }
    out.push_str(&ResetColor.to_string());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_match_stops() {
        let g = Gradient::new(vec![(0, 0, 0), (255, 255, 255)]);
        assert_eq!(g.sample(0.0), (0, 0, 0));
        assert_eq!(g.sample(1.0), (255, 255, 255));
        // midpoint is roughly grey
        let (r, _, _) = g.sample(0.5);
        assert!((120..=135).contains(&r));
    }

    #[test]
    fn single_stop_is_constant() {
        let g = Gradient::new(vec![(10, 20, 30)]);
        assert_eq!(g.sample(0.0), (10, 20, 30));
        assert_eq!(g.sample(1.0), (10, 20, 30));
    }

    #[test]
    fn empty_stops_fall_back() {
        let g = Gradient::new(vec![]);
        assert_eq!(g.sample(0.5), (255, 255, 255));
    }

    #[test]
    fn paint_disabled_is_identity() {
        let g = Gradient::rainbow();
        assert_eq!(paint("EUREKA", &g, false), "EUREKA");
    }

    #[test]
    fn paint_enabled_keeps_text_and_adds_ansi() {
        let g = Gradient::rainbow();
        let painted = paint("EUREKA", &g, true);
        // Every original character is still present, and ANSI codes were added.
        for ch in "EUREKA".chars() {
            assert!(painted.contains(ch), "missing {ch}");
        }
        assert!(painted.len() > "EUREKA".len());
        // The visible width ignores the ANSI codes.
        assert_eq!(crate::ui::text::visible_width(&painted), 6);
    }
}
