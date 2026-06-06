//! Abstraction over the terminal so that animations stay testable.
//!
//! The [`crate::ui::animate::Player`] only talks to this [`Surface`] trait.
//! In production, [`TermSurface`] writes to stdout via crossterm; in tests
//! [`BufferSurface`] collects the output in memory.

use std::io::{self, Write};

/// Minimal terminal operations that the animation player needs.
pub trait Surface {
    /// Writes `s` without a line break.
    fn write_str(&mut self, s: &str) -> io::Result<()>;

    /// Writes `s` with a trailing line break.
    fn write_line(&mut self, s: &str) -> io::Result<()> {
        self.write_str(s)?;
        self.write_str("\n")
    }

    /// Flushes buffered output.
    fn flush(&mut self) -> io::Result<()>;

    /// Moves the cursor `n` lines up (0 = no movement).
    fn move_up(&mut self, n: u16) -> io::Result<()>;

    /// Clears everything from the cursor to the end of the screen.
    fn clear_below(&mut self) -> io::Result<()>;

    /// Hides the cursor (e.g. during an animation).
    fn hide_cursor(&mut self) -> io::Result<()>;

    /// Shows the cursor again.
    fn show_cursor(&mut self) -> io::Result<()>;

    /// Current terminal width in columns.
    fn width(&self) -> u16;
}

/// Writes to a real terminal (via crossterm cursor control).
pub struct TermSurface<W: Write> {
    writer: W,
    width: u16,
}

impl TermSurface<io::Stdout> {
    /// Surface over stdout with detected terminal width (fallback: 80).
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
    /// Surface over an arbitrary writer with a fixed width.
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

/// Collects all text output in memory – for tests. Cursor/clear operations
/// are no-ops.
#[derive(Debug, Default)]
pub struct BufferSurface {
    /// The collected (written) output.
    pub out: String,
    width: u16,
}

impl BufferSurface {
    /// New buffer with width `width`.
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
