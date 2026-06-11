//! `rubberduck doctor` — a quick, read-only environment and configuration check.
//!
//! It answers "is my rubberduck set up the way I think?" at a glance: the active
//! version and language, the resolved theme, whether colour and animations will
//! actually run in this terminal, where the config and data live, and how much
//! content is bundled. It writes nothing and is fully localized.

use crate::paths;
use crate::questions;
use crate::tips;
use crate::ui::gradient::Gradient;
use crate::ui::theme::Styler;
use crate::ui::Ui;
use anyhow::Result;

/// Prints the diagnostics for the current environment and settings.
pub fn run(ui: &mut Ui) -> Result<()> {
    let tr = ui.tr();
    let lang = tr.lang();

    // Gather terminal/colour facts before borrowing the styler for printing, so
    // the immutable `ui` reads don't collide with the later `ui.styler()` borrow.
    let color_on = ui.styler().enabled();
    let theme_name = ui.styler().theme().name;
    let anim_on = ui.animating();
    let interactive = ui.is_interactive();

    let topics = questions::embedded(lang).map(|p| p.len()).unwrap_or(0);
    let tip_count = tips::embedded(lang).map(|p| p.len()).unwrap_or(0);
    let config_path = paths::config_file()?;
    let data_dir = paths::data_dir()?;
    let config_exists = config_path.exists();

    ui.gradient_banner(tr.doctor_header(), &Gradient::mint());
    let st = ui.styler();

    let on_off = |on: bool| {
        if on {
            tr.doctor_enabled()
        } else {
            tr.doctor_disabled()
        }
    };

    row(st, tr.doctor_version(), env!("CARGO_PKG_VERSION"));
    row(
        st,
        tr.doctor_language(),
        &format!("{} ({})", lang.code(), lang.label()),
    );
    row(st, tr.doctor_theme(), theme_name);
    row(st, tr.doctor_color(), on_off(color_on));
    row(st, tr.doctor_animations(), on_off(anim_on));
    row(
        st,
        tr.doctor_terminal(),
        if interactive {
            tr.doctor_interactive()
        } else {
            tr.doctor_plain()
        },
    );
    let cfg_status = if config_exists {
        tr.doctor_config_ok()
    } else {
        tr.doctor_config_missing()
    };
    row(
        st,
        tr.doctor_config(),
        &format!("{} ({})", config_path.display(), cfg_status),
    );
    row(st, tr.doctor_data(), &data_dir.display().to_string());

    // Self-describing content lines (the value already names what it counts).
    println!(
        "  {} {}",
        st.accent("•"),
        st.text(&tr.doctor_questions(topics, lang.code()))
    );
    println!(
        "  {} {}",
        st.accent("•"),
        st.text(&tr.doctor_tips(tip_count, lang.code()))
    );
    Ok(())
}

/// Prints a single `• Label: value` diagnostic row.
fn row(st: &Styler, label: &str, value: &str) {
    println!("  {} {}: {}", st.accent("•"), st.text(label), st.dim(value));
}
