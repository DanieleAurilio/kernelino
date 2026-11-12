use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

pub fn clear_terminal(out: &mut impl Write) -> io::Result<()> {
    execute!(out, Clear(ClearType::All), MoveTo(0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_terminal_clears_and_homes_the_cursor() {
        let mut out = Vec::new();
        clear_terminal(&mut out).unwrap();
        assert_eq!(out, b"\x1b[2J\x1b[1;1H");
    }
}
