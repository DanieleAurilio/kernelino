use std::io::{self, Write};
use lazy_static::lazy_static;
use std::sync::RwLock;
use crossterm::{
    self,
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

use crate::vfs::{init_vfs, Vfs};

enum ShellCommand {
    Exit,
    Help,
    Clear,
    NewLine,
    Pwd,
    Cd(String),
    MkDir(String),
    Ls,
    Rm(String),
    Touch(String)
}

// Initialize the virtual file system
lazy_static!(
    static ref VFS: RwLock<Vfs> = RwLock::new(init_vfs());
);

impl ShellCommand {
    fn from_str(input: &str) -> Option<Self> {
        match input {
            "" => Some(Self::NewLine),
            "exit" => Some(Self::Exit),
            "help" => Some(Self::Help),
            "clear" => Some(Self::Clear),
            "pwd" => Some(Self::Pwd),
            "ls" => Some(Self::Ls),
            _ if input.starts_with("cd") => Some(Self::Cd(input.trim_start_matches("cd ").to_string())),
            _ if input.starts_with("mkdir") => Some(Self::MkDir(input.trim_start_matches("mkdir ").to_string())),
            _ if input.starts_with("rm") => Some(Self::Rm(input.trim_start_matches("rm ").to_string())),
            _ if input.starts_with("touch") => Some(Self::Touch(input.trim_start_matches("touch ").to_string())),
            _ => None,
        }
    }

    fn execute(&self) {
        match self {
            Self::NewLine => cmd_newline(),
            Self::Exit => cmd_exit(),
            Self::Help => cmd_help(),
            Self::Clear => cmd_clear(),
            Self::Pwd => cmd_pwd(),
            Self::Cd(cd) => cmd_cd(cd),
            Self::MkDir(dir) => cmd_add_directory( dir),
            Self::Ls => cmd_ls(),
            Self::Rm(path) => cmd_rm(path),
            Self::Touch(filename) => cmd_touch(filename),
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
            None => println!("Unknown command: {}", input),
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
    println!("  clear - Clear the screen");
    println!("  pwd - Print the current working directory");
    println!("  cd <path> - Change the current working directory");
    println!("  mkdir <name> - Create a new directory");
}

fn cmd_clear() {
    io::stdout().execute(Clear(ClearType::All)).unwrap();
    io::stdout().execute(MoveTo(0, 0)).unwrap();
}

fn cmd_newline() {
    print!("");
}

fn cmd_pwd() {
    let vfs = VFS.read().unwrap().clone();
    vfs.pwd();
}

fn cmd_cd(path: &str) {
    let mut vfs = VFS.write().unwrap();
    vfs.change_dir(path);
}

fn cmd_add_directory(name: &str) {
    let mut vfs = VFS.write().unwrap();
    vfs.add_directory_recursive(name);
}

fn cmd_ls() {
    let mut vfs = VFS.write().unwrap();
    vfs.list();
}

fn cmd_rm(path: &str) {
    let mut vfs = VFS.write().unwrap();
    vfs.remove(path);
}

fn cmd_touch(filename: &str) {
    let mut vfs = VFS.write().unwrap();
    vfs.touch(filename);
}