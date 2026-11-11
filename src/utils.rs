use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

pub fn clear_terminal(out: &mut impl Write) -> io::Result<()> {
    execute!(out, Clear(ClearType::All), MoveTo(0, 0))
}
