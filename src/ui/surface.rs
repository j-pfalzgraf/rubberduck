//! Abstraktion über das Terminal, damit Animationen testbar bleiben.
//!
//! Der [`crate::ui::animate::Player`] spricht nur dieses [`Surface`]-Trait an.
//! In Produktion schreibt [`TermSurface`] via crossterm auf stdout; in Tests
//! sammelt [`BufferSurface`] die Ausgabe im Speicher.

use std::io::{self, Write};

/// Minimale Terminal-Operationen, die der Animations-Player benötigt.
pub trait Surface {
    /// Schreibt `s` ohne Zeilenumbruch.
    fn write_str(&mut self, s: &str) -> io::Result<()>;

    /// Schreibt `s` mit abschließendem Zeilenumbruch.
    fn write_line(&mut self, s: &str) -> io::Result<()> {
        self.write_str(s)?;
        self.write_str("\n")
    }

    /// Schreibt gepufferte Ausgaben raus.
    fn flush(&mut self) -> io::Result<()>;

    /// Bewegt den Cursor `n` Zeilen nach oben (0 = keine Bewegung).
    fn move_up(&mut self, n: u16) -> io::Result<()>;

    /// Löscht alles vom Cursor bis zum Bildschirmende.
    fn clear_below(&mut self) -> io::Result<()>;

    /// Versteckt den Cursor (z. B. während einer Animation).
    fn hide_cursor(&mut self) -> io::Result<()>;

    /// Zeigt den Cursor wieder an.
    fn show_cursor(&mut self) -> io::Result<()>;

    /// Aktuelle Terminalbreite in Spalten.
    fn width(&self) -> u16;
}

/// Schreibt auf ein echtes Terminal (per crossterm-Cursorsteuerung).
pub struct TermSurface<W: Write> {
    writer: W,
    width: u16,
}

impl TermSurface<io::Stdout> {
    /// Surface über stdout mit erkannter Terminalbreite (Fallback: 80).
    #[must_use]
    pub fn stdout() -> Self {
        let width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);
        Self {
            writer: io::stdout(),
            width,
        }
    }
}

impl<W: Write> TermSurface<W> {
    /// Surface über einen beliebigen Writer mit fixer Breite.
    pub fn new(writer: W, width: u16) -> Self {
        Self { writer, width }
    }
}

impl<W: Write> Surface for TermSurface<W> {
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_all(s.as_bytes())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    fn move_up(&mut self, n: u16) -> io::Result<()> {
        if n > 0 {
            crossterm::queue!(self.writer, crossterm::cursor::MoveUp(n))?;
        }
        Ok(())
    }

    fn clear_below(&mut self) -> io::Result<()> {
        crossterm::queue!(
            self.writer,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
        )?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::cursor::Hide)?;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        crossterm::queue!(self.writer, crossterm::cursor::Show)?;
        Ok(())
    }

    fn width(&self) -> u16 {
        self.width
    }
}

/// Sammelt alle Textausgaben im Speicher – für Tests. Cursor-/Clear-Operationen
/// sind No-ops.
#[derive(Debug, Default)]
pub struct BufferSurface {
    /// Die gesammelte (geschriebene) Ausgabe.
    pub out: String,
    width: u16,
}

impl BufferSurface {
    /// Neuer Puffer mit Breite `width`.
    #[must_use]
    pub fn new(width: u16) -> Self {
        Self {
            out: String::new(),
            width,
        }
    }
}

impl Surface for BufferSurface {
    fn write_str(&mut self, s: &str) -> io::Result<()> {
        self.out.push_str(s);
        Ok(())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn move_up(&mut self, _n: u16) -> io::Result<()> {
        Ok(())
    }
    fn clear_below(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn width(&self) -> u16 {
        self.width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_records_writes() {
        let mut s = BufferSurface::new(40);
        s.write_line("a").unwrap();
        s.write_str("b").unwrap();
        assert_eq!(s.out, "a\nb");
        assert_eq!(s.width(), 40);
    }
}
