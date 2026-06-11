//! The animated guided tour shown by `rubberduck demo`.
//!
//! It exercises every animation (gradient banner, fluid swim-in entrance,
//! typewriter, quack, all spinner styles, all moods, theme previews and the
//! confetti celebration) so users can see what rubberduck can do. Like
//! everything else it degrades cleanly to static output without a terminal and
//! is fully localized.

use crate::ui::duck::Mood;
use crate::ui::gradient::Gradient;
use crate::ui::spinner::SpinnerStyle;
use crate::ui::Ui;
use anyhow::Result;

/// Moods shown, in order, during the demo.
const SHOWCASE: [Mood; 9] = [
    Mood::Idle,
    Mood::Thinking,
    Mood::Curious,
    Mood::Confused,
    Mood::Reading,
    Mood::Happy,
    Mood::Proud,
    Mood::Surprised,
    Mood::Sleeping,
];

/// Plays the guided tour of rubberduck's animations.
pub fn run(ui: &mut Ui) -> Result<()> {
    let tr = ui.tr();

    ui.gradient_banner(tr.demo_title(), &Gradient::rainbow());
    println!();

    ui.swim_in(Mood::Idle)?;
    ui.duck_says(tr.demo_intro(), Mood::Listening)?;
    ui.quack(Mood::Happy)?;
    ui.thinking(tr.pondering(), 12)?;

    println!("\n{}", ui.styler().accent(tr.demo_section_moods()));
    for mood in SHOWCASE {
        ui.duck_says(tr.mood_label(mood), mood)?;
    }

    println!("\n{}", ui.styler().accent(tr.demo_section_spinners()));
    for style in SpinnerStyle::ALL {
        ui.spinner_showcase(style)?;
    }

    println!("\n{}", ui.styler().accent(tr.demo_section_gradients()));
    ui.gradient_previews();

    println!("\n{}", ui.styler().accent(tr.demo_section_themes()));
    ui.theme_previews();

    println!();
    ui.celebrate()?;
    ui.duck_says(tr.demo_done(), Mood::Happy)?;
    Ok(())
}
