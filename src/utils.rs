use std::io::stdout;

use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

pub fn is_unix_symbol(s: &str) -> bool {
    const PROTECTED_SYMBOL: [&str; 3] = ["/", ".", ".."];
    PROTECTED_SYMBOL.contains(&s)
}

pub fn clear_terminal() {
    stdout().execute(Clear(ClearType::All)).unwrap();
    stdout().execute(MoveTo(0, 0)).unwrap();
}
