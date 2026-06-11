//! Reusable horizontal bar charts and inline meters.
//!
//! [`GrowingBars`] is a generic, animated horizontal bar chart: it grows every
//! bar from zero to its target length over a handful of frames. The `stats`
//! view and any future chart build on it instead of re-implementing the layout,
//! keeping the bar maths in exactly one place (DRY).
//!
//! [`meter`] renders a single static bar for inline use (e.g. the `stats`
//! solve-rate gauge) without any animation.

use crate::ui::animate::{Animation, Frame};
use crate::ui::gradient::{self, Gradient};
use crate::ui::text;
use crate::ui::theme::Styler;
use std::time::Duration;

/// Frames a [`GrowingBars`] chart grows over.
const FRAMES: usize = 12;
/// Width, in cells, of a full bar.
const BAR_WIDTH: usize = 24;

/// One labelled value in a [`GrowingBars`] chart.
#[derive(Debug, Clone)]
pub struct BarRow {
    /// The row label (printed left of the bar).
    pub label: String,
    /// The numeric value the bar length encodes.
    pub value: usize,
    /// Optional trailing caption (printed right of the bar); the value when empty.
    pub caption: Option<String>,
}

impl BarRow {
    /// A row with just a label and a value (the value doubles as the caption).
    #[must_use]
    pub fn new(label: impl Into<String>, value: usize) -> Self {
        Self {
            label: label.into(),
            value,
            caption: None,
        }
    }

    /// A row with an explicit trailing caption (e.g. `"3 (67%)"`).
    #[must_use]
    pub fn with_caption(
        label: impl Into<String>,
        value: usize,
        caption: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            value,
            caption: Some(caption.into()),
        }
    }
}

/// Number of filled cells for `value`/`max` over a `width`-wide bar, scaled by
/// the animation `progress` (`0.0..=1.0`). Saturates safely when `max == 0`.
#[must_use]
pub fn cells(value: usize, max: usize, width: usize, progress: f32) -> usize {
    let full = (value * width).checked_div(max).unwrap_or(0);
    (full as f32 * progress.clamp(0.0, 1.0)).round() as usize
}

/// A static single bar of `width` cells filled to `ratio` (`0.0..=1.0`),
/// gradient-painted when `enabled`. Used for the `stats` solve-rate gauge.
#[must_use]
pub fn meter(ratio: f32, width: usize, gradient: &Gradient, enabled: bool) -> String {
    let filled = (width as f32 * ratio.clamp(0.0, 1.0)).round() as usize;
    let bar = gradient::paint(&"█".repeat(filled), gradient, enabled);
    let rest = "░".repeat(width.saturating_sub(filled));
    format!("{bar}{rest}")
}

/// An animated horizontal bar chart: every bar grows from zero to full.
pub struct GrowingBars {
    rows: Vec<BarRow>,
    max: usize,
    styler: Styler,
    gradient: Gradient,
    label_width: usize,
}

impl GrowingBars {
    /// New chart from its rows, a styler and a gradient. The label column is
    /// sized to the longest label (clamped) and bars are scaled to the largest
    /// value.
    #[must_use]
    pub fn new(rows: Vec<BarRow>, styler: Styler, gradient: Gradient) -> Self {
        let max = rows.iter().map(|r| r.value).max().unwrap_or(0);
        let label_width = rows
            .iter()
            .map(|r| text::display_width(&r.label))
            .max()
            .unwrap_or(0)
            .clamp(4, 16);
        Self {
            rows,
            max,
            styler,
            gradient,
            label_width,
        }
    }

    /// Renders one row at the given fill `progress`.
    fn row_line(&self, row: &BarRow, progress: f32) -> String {
        let len = cells(row.value, self.max, BAR_WIDTH, progress);
        let bar = gradient::paint(&"█".repeat(len), &self.gradient, self.styler.enabled());
        // Fit the label to an exact column width so bars stay aligned even for
        // long or wide-character labels; pass the caption by reference to dim.
        let label = text::fit(&row.label, self.label_width);
        let caption = match &row.caption {
            Some(c) => self.styler.dim(c),
            None => self.styler.dim(&row.value.to_string()),
        };
        format!("  {label} {bar} {caption}")
    }
}

impl Animation for GrowingBars {
    fn frame_count(&self) -> usize {
        if self.rows.is_empty() {
            0
        } else {
            FRAMES
        }
    }

    fn frame(&self, i: usize) -> Frame {
        let progress = (i + 1) as f32 / FRAMES as f32;
        let lines = self
            .rows
            .iter()
            .map(|row| self.row_line(row, progress))
            .collect();
        Frame::new(lines)
    }

    fn delay(&self, _i: usize) -> Duration {
        Duration::from_millis(55)
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
    fn cells_scale_with_value_and_progress() {
        assert_eq!(cells(10, 10, 20, 1.0), 20);
        assert_eq!(cells(5, 10, 20, 1.0), 10);
        assert_eq!(cells(10, 10, 20, 0.5), 10);
        // max == 0 must not divide by zero.
        assert_eq!(cells(3, 0, 20, 1.0), 0);
    }

    #[test]
    fn meter_fills_proportionally() {
        let g = Gradient::ocean();
        assert_eq!(meter(0.0, 10, &g, false), "░░░░░░░░░░");
        assert_eq!(meter(1.0, 10, &g, false), "██████████");
        assert_eq!(meter(0.5, 10, &g, false), "█████░░░░░");
    }

    #[test]
    fn growing_bars_reach_full_length_on_the_last_frame() {
        let rows = vec![BarRow::new("logic", 3), BarRow::new("api", 1)];
        let chart = GrowingBars::new(rows, styler(), Gradient::ocean());
        assert_eq!(chart.frame_count(), 12);
        let last = chart.frame(chart.frame_count() - 1).lines.join("\n");
        assert!(last.contains("logic") && last.contains("api"));
        assert!(last.contains('█'), "busiest row should have a full bar");
    }

    #[test]
    fn empty_chart_has_no_frames() {
        let chart = GrowingBars::new(Vec::new(), styler(), Gradient::ocean());
        assert_eq!(chart.frame_count(), 0);
    }

    #[test]
    fn captions_override_the_value() {
        let rows = vec![BarRow::with_caption("logic", 3, "3 (67%)")];
        let chart = GrowingBars::new(rows, styler(), Gradient::ocean());
        let line = chart.frame(chart.frame_count() - 1).lines.join("\n");
        assert!(line.contains("3 (67%)"));
    }

    #[test]
    fn over_long_labels_are_clipped_so_bars_stay_aligned() {
        let rows = vec![
            BarRow::new("a-really-very-long-topic-name", 2),
            BarRow::new("api", 1),
        ];
        let chart = GrowingBars::new(rows, styler(), Gradient::ocean());
        let frame = chart.frame(chart.frame_count() - 1);
        // Both labels are clipped/padded to the same column width, so the bars
        // (the first '█') start at the same *display column* on every row — even
        // though the byte offset differs (the clipped label ends in a 3-byte '…').
        let bar_col = |line: &str| line.find('█').map(|idx| text::display_width(&line[..idx]));
        let cols: Vec<_> = frame.lines.iter().filter_map(|l| bar_col(l)).collect();
        assert_eq!(cols.len(), 2);
        assert_eq!(
            cols[0], cols[1],
            "bars must align regardless of label length"
        );
    }
}
