use std::io::{self, Write};

use crossterm::{
    self, cursor::MoveTo, terminal::{Clear, ClearType}, ExecutableCommand
};

enum ShellCommand {
    Exit,
    Help,
    Clear
}

impl ShellCommand {
    fn from_str(input: &str) -> Option<Self> {
        match input {
            "exit" => Some(Self::Exit),
            "help" => Some(Self::Help),
            "clear" => Some(Self::Clear),
            _ => None,
        }
    }

    fn execute(&self) {
        match self {
            Self::Exit => cmd_exit(),
            Self::Help => cmd_help(),
            Self::Clear => cmd_clear()
        }
    }
}

pub fn run() {
    // Setting up the terminal
    cmd_clear();

    loop {
        print!("kernelino> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();

        match ShellCommand::from_str(input) {
            Some(cmd) => cmd.execute(),
            None => println!("Unknown command: {}", input)
        }
    }
}

fn cmd_exit() {
    println!("Goodbye!");
    std::process::exit(0);
}

fn cmd_help() {
    println!("Available commands:");
    println!("  exit - Exit the shell");
    println!("  help - Display this help message");
}

fn cmd_clear() {
    io::stdout().execute(Clear(ClearType::All)).unwrap();
    io::stdout().execute(MoveTo(0, 0)).unwrap();
}