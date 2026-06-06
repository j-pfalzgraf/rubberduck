//! The `rubberduck stats` view: aggregate session metrics with an animated bar chart.

use crate::history::{self, Aggregate, TopicAggregate};
use crate::session::format_duration;
use crate::ui::animate::{Animation, Frame};
use crate::ui::gradient::{self, Gradient};
use crate::ui::theme::Styler;
use crate::ui::Ui;
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::Duration;

/// Shows aggregate statistics, clears the history (`reset`) or prints JSON (`json`).
pub fn show(ui: &mut Ui, reset: bool, json: bool) -> Result<()> {
    let tr = ui.tr();
    if reset {
        history::clear()?;
        println!("{}", ui.styler().dim(tr.stats_cleared()));
        return Ok(());
    }

    let agg = history::aggregate(&history::load_all()?);

    if json {
        let view = JsonView {
            sessions: agg.sessions,
            solved: agg.solved,
            solve_rate: agg.solve_rate(),
            avg_total_seconds: agg.avg_total_seconds(),
            avg_solution_seconds: agg.avg_solution_seconds(),
            per_topic: &agg.per_topic,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&view).context("Could not serialize stats")?
        );
        return Ok(());
    }

    ui.gradient_banner(tr.stats_header(), &Gradient::ocean());

    if agg.is_empty() {
        println!("{}", ui.styler().dim(tr.stats_empty()));
        return Ok(());
    }

    {
        let st = ui.styler();
        println!("  {} {}", st.accent("•"), tr.stats_sessions(agg.sessions));
        println!(
            "  {} {}",
            st.accent("•"),
            tr.stats_solved(agg.solved, agg.sessions, agg.solve_rate())
        );
        println!(
            "  {} {}",
            st.accent("•"),
            tr.stats_avg_session(&format_duration(agg.avg_total_seconds()))
        );
        if agg.solution_count > 0 {
            println!(
                "  {} {}",
                st.accent("•"),
                tr.stats_avg_solution(&format_duration(agg.avg_solution_seconds()))
            );
        }
        println!("\n{}", st.accent(tr.stats_by_topic()));
    }

    let chart = BarChart::new(&agg, *ui.styler(), Gradient::ocean());
    ui.play(&chart)?;
    Ok(())
}

/// Machine-readable stats view for `stats --json`.
#[derive(Serialize)]
struct JsonView<'a> {
    sessions: usize,
    solved: usize,
    solve_rate: u32,
    avg_total_seconds: u64,
    avg_solution_seconds: u64,
    per_topic: &'a BTreeMap<String, TopicAggregate>,
}

/// An animated horizontal bar chart of sessions per topic (bars grow in).
struct BarChart {
    rows: Vec<(String, usize)>,
    max: usize,
    styler: Styler,
    gradient: Gradient,
    frames: usize,
    bar_width: usize,
}

impl BarChart {
    fn new(agg: &Aggregate, styler: Styler, gradient: Gradient) -> Self {
        let rows: Vec<(String, usize)> = agg
            .per_topic
            .iter()
            .map(|(name, t)| (name.clone(), t.sessions))
            .collect();
        let max = rows.iter().map(|(_, v)| *v).max().unwrap_or(0);
        Self {
            rows,
            max,
            styler,
            gradient,
            frames: 12,
            bar_width: 24,
        }
    }
}

impl Animation for BarChart {
    fn frame_count(&self) -> usize {
        if self.rows.is_empty() {
            0
        } else {
            self.frames
        }
    }

    fn frame(&self, i: usize) -> Frame {
        let progress = (i + 1) as f32 / self.frames as f32;
        let lines = self
            .rows
            .iter()
            .map(|(name, value)| {
                let full = (value * self.bar_width).checked_div(self.max).unwrap_or(0);
                let len = (full as f32 * progress).round() as usize;
                let bar = gradient::paint(&"█".repeat(len), &self.gradient, self.styler.enabled());
                format!(
                    "  {:<8} {} {}",
                    name,
                    bar,
                    self.styler.dim(&value.to_string())
                )
            })
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
    use crate::history::TopicAggregate;
    use crate::ui::theme::Theme;

    #[test]
    fn bar_chart_renders_topics_with_full_bars_at_end() {
        let mut agg = Aggregate {
            sessions: 4,
            ..Aggregate::default()
        };
        agg.per_topic.insert(
            "logic".into(),
            TopicAggregate {
                sessions: 3,
                solved: 2,
            },
        );
        agg.per_topic.insert(
            "api".into(),
            TopicAggregate {
                sessions: 1,
                solved: 0,
            },
        );

        let chart = BarChart::new(&agg, Styler::new(Theme::CLASSIC, false), Gradient::ocean());
        assert_eq!(chart.frame_count(), 12);
        let last = chart.frame(chart.frame_count() - 1).lines.join("\n");
        assert!(last.contains("logic") && last.contains("api"));
        assert!(
            last.contains('█'),
            "the busiest topic should have a full bar"
        );
    }

    #[test]
    fn empty_chart_has_no_frames() {
        let chart = BarChart::new(
            &Aggregate::default(),
            Styler::new(Theme::CLASSIC, false),
            Gradient::ocean(),
        );
        assert_eq!(chart.frame_count(), 0);
    }
}
