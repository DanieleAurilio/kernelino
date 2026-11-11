mod editor;
mod kernel;
mod lua;
mod shell;
mod utils;
mod vfs;
mod vmm;
mod vpm;

use std::io;

fn main() -> io::Result<()> {
    let mut out = io::stdout();
    utils::clear_terminal(&mut out)?;
    shell::Shell::new().run(&mut io::stdin().lock(), &mut out)
}
