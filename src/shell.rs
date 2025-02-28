use crate::kpm::Kpm;
use crate::utils;
use lazy_static::lazy_static;
use std::io::{self, Write};
use std::sync::RwLock;

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
    Touch(String),
    WriteFile(String),
    ReadFile(String),
    Top,
    KpmInstall(String),
    KpmList(),
    KpmExec(String),
}

// Initialize the virtual file system
lazy_static! {
    pub static ref VFS: RwLock<Vfs> = RwLock::new(init_vfs());
    pub static ref kpm: RwLock<Kpm> = RwLock::new(Kpm::init());
}

impl ShellCommand {
    fn from_str(input: &str) -> Option<Self> {
        match input {
            "" => Some(Self::NewLine),
            "exit" => Some(Self::Exit),
            "help" => Some(Self::Help),
            "clear" => Some(Self::Clear),
            "pwd" => Some(Self::Pwd),
            "ls" => Some(Self::Ls),
            "top" => Some(Self::Top),
            _ if input.starts_with("cd") => {
                Some(Self::Cd(input.trim_start_matches("cd ").to_string()))
            }
            _ if input.starts_with("mkdir") => {
                Some(Self::MkDir(input.trim_start_matches("mkdir ").to_string()))
            }
            _ if input.starts_with("rm") => {
                Some(Self::Rm(input.trim_start_matches("rm ").to_string()))
            }
            _ if input.starts_with("touch") => {
                Some(Self::Touch(input.trim_start_matches("touch ").to_string()))
            }
            _ if input.starts_with("write") => Some(Self::WriteFile(
                input.trim_start_matches("write ").to_string(),
            )),
            _ if input.starts_with("read") => Some(Self::ReadFile(
                input.trim_start_matches("read ").to_string(),
            )),
            _ if input.starts_with("kpm install") => Some(Self::KpmInstall(
                input.trim_start_matches("kpm install ").to_string(),
            )),
            _ if input.starts_with("kpm list") => Some(Self::KpmList()),
            _ if input.starts_with("kpm exec") => Some(Self::KpmExec(
                input.trim_start_matches("kpm exec ").to_string(),
            )),
            _ => None,
        }
    }

    async fn execute(&self) {
        match self {
            Self::NewLine => cmd_newline(),
            Self::Exit => cmd_exit(),
            Self::Help => cmd_help(),
            Self::Clear => cmd_clear(),
            Self::Pwd => cmd_pwd(),
            Self::Cd(cd) => cmd_cd(cd),
            Self::MkDir(dir) => cmd_add_directory(dir),
            Self::Ls => cmd_ls(),
            Self::Rm(path) => cmd_rm(path),
            Self::Touch(filename) => cmd_touch(filename),
            Self::WriteFile(filename) => cmd_write_file(filename),
            Self::ReadFile(filename) => cmd_read_file(filename),
            Self::Top => cmd_top(),
            Self::KpmInstall(package_name) => {
                cmd_kpm_install(package_name).await;
            }
            Self::KpmList() => cmd_kpm_list(),
            Self::KpmExec(package_name) => {
                cmd_kpm_exec(package_name).await;
            }
        }
    }
}

pub async fn run() {
    // Initialize the base file system
    init_base_fs();

    // Setting up the terminal
    cmd_clear();

    loop {
        print!("kernelino> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();

        match ShellCommand::from_str(input) {
            Some(cmd) => cmd.execute().await,
            None => println!("Unknown command: {}", input),
        }
    }
}

fn init_base_fs() {
    cmd_add_directory("bin");
    cmd_add_directory("tmp");
    cmd_touch(".env");
}

async fn cmd_kpm_install(package_name: &str) {
    let mut mut_kpm = kpm.write().unwrap();

    let package_download_info = mut_kpm.download(package_name).await;
    let package_bytes = package_download_info.0;
    let package_filename = package_download_info.1;
    let package_version = package_download_info.2;
    if package_bytes.is_none() || package_filename.is_none() || package_version.is_none() {
        return;
    }

    let package_fullname_tmp = package_filename.unwrap();
    let vfs = VFS.write().unwrap();
    mut_kpm.install(
        vfs,
        package_bytes.as_ref().unwrap(),
        package_fullname_tmp.as_str(),
        package_version.as_ref().unwrap(),
    );

    println!("Package installed successfully!");

    return;
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
    println!("  touch <filename> - Create a new file");
    println!("  write <filename> - Write file content");
    println!("  read <filename> - Read file content");
    println!("  ls - List directory contents");
    println!("  rm <path> - Remove a file or directory");
    println!("  top - Show the processes");
    println!("  kpm install <package> - Install a package");
    println!("  kpm list - List all available packages");
    println!("  kpm exec <package> - Execute a package");
}

fn cmd_clear() {
    utils::clear_terminal();
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

fn cmd_write_file(filename: &str) {
    let mut vfs = VFS.write().unwrap();
    vfs.write_file(filename, None, None);
}

fn cmd_read_file(filename: &str) {
    let mut vfs = VFS.read().unwrap().clone();
    vfs.read_file(filename);
}

fn cmd_top() {
    let vfs = VFS.read().unwrap().clone();
    vfs.vpm.show_processes()
}

fn cmd_kpm_list() {
    let kpm_read = kpm.read().unwrap();
    kpm_read.list();
}

async fn cmd_kpm_exec(package_name: &str) {
    let kpm_read = kpm.read().unwrap();
    let vfs = VFS.write().unwrap();
    kpm_read.execute(vfs, package_name).await;
}
