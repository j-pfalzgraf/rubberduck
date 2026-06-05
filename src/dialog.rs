//! Interaktiver Frage-Dialog.

use crate::duck;
use crate::questions::QuestionPool;
use crate::session::Transcript;
use anyhow::Result;
use chrono::Local;
use dialoguer::Input;
use std::io::ErrorKind;

/// Führt eine komplette Session zum gewählten Thema und liefert das Transkript.
pub fn run_session(topic: &str, quiet: bool, pool: &QuestionPool) -> Result<Transcript> {
    // Thema zuerst prüfen: schlägt früh fehl, bevor irgendetwas ausgegeben wird.
    let questions = pool.questions_for(topic)?;

    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let started_at = now.format("%Y-%m-%d %H:%M").to_string();

    greet(quiet, topic);

    let mut entries = Vec::with_capacity(questions.len());
    for question in questions {
        if quiet {
            println!("\n❓ {question}");
        } else {
            duck::print_duck_says(question);
        }

        match Input::<String>::new()
            .with_prompt("  Du")
            .allow_empty(true)
            .interact_text()
        {
            Ok(answer) => entries.push((question.clone(), answer)),
            // Abbruch (ESC/EOF) oder fehlende Terminal-Eingabe (Pipe/CI):
            // Session sauber beenden und das bisher Gesagte behalten.
            Err(dialoguer::Error::IO(e))
                if matches!(
                    e.kind(),
                    ErrorKind::Interrupted | ErrorKind::NotConnected | ErrorKind::UnexpectedEof
                ) =>
            {
                println!("\nAbgebrochen – bis zum nächsten Quaken!");
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }

    farewell(quiet);

    Ok(Transcript {
        topic: topic.to_string(),
        date,
        started_at,
        entries,
    })
}

fn greet(quiet: bool, topic: &str) {
    let msg = format!(
        "Hallo! Ich bin deine Debugging-Ente (Thema: {topic}). Erklär mir dein \
         Problem Schritt für Schritt – oft fällt der Groschen schon beim Reden."
    );
    if quiet {
        println!("🦆 {msg}");
    } else {
        duck::print_duck_says(&msg);
    }
}

fn farewell(quiet: bool) {
    let msg = "Danke fürs Erklären! Wenn der Groschen gefallen ist, hat die Ente \
               ihren Job getan.";
    if quiet {
        println!("\n🦆 {msg}");
    } else {
        println!();
        duck::print_duck_says(msg);
    }
}
