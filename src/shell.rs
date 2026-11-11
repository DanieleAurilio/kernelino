use std::error::Error;
use std::io::{self, BufRead, Write};
use std::panic::{self, AssertUnwindSafe};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::kernel::Kernel;
use crate::lua::{Limits, LuaProcess, Step};
use crate::vfs::{Node, VfsError};
use crate::{editor, utils};

const TOTAL_MEMORY: u64 = 4 * 1024 * 1024 * 1024;
const MEGABYTE: u64 = 1024 * 1024;

const LUA_LIMITS: Limits = Limits {
    memory: 64 * MEGABYTE as usize,
    instructions: 1_000_000_000,
};

const HELP: &[(&str, &str)] = &[
    ("exit", "Exit the shell"),
    ("help", "Display this help message"),
    ("clear", "Clear the screen"),
    ("pwd", "Print the current working directory"),
    ("cd [path]", "Change the current working directory"),
    ("ls [path]", "List directory contents"),
    ("mkdir <path>", "Create a directory and its missing parents"),
    ("touch <path>", "Create an empty file"),
    (
        "write <path>",
        "Write a file with the editor, creating it if needed",
    ),
    ("read <path>", "Print file content"),
    (
        "rm <path>",
        "Remove a file, or a directory with its content",
    ),
    ("top", "Show the processes"),
    ("lua <path>", "Run a Lua 5.4 script in the sandbox"),
];

type CommandResult = Result<(), Box<dyn Error>>;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Exit,
    Help,
    Clear,
    Pwd,
    Top,
    Cd(String),
    Ls(String),
    MkDir(String),
    Touch(String),
    Write(String),
    Read(String),
    Rm(String),
    Lua(String),
}

impl Command {
    /// `Ok(None)` for an empty line.
    fn parse(line: &str) -> Result<Option<Self>, String> {
        let mut words = line.split_whitespace();
        let Some(name) = words.next() else {
            return Ok(None);
        };
        let args: Vec<&str> = words.collect();

        let command = match (name, args.as_slice()) {
            ("exit", []) => Self::Exit,
            ("help", []) => Self::Help,
            ("clear", []) => Self::Clear,
            ("pwd", []) => Self::Pwd,
            ("top", []) => Self::Top,
            ("cd", []) => Self::Cd(String::from("/")),
            ("cd", [path]) => Self::Cd(path.to_string()),
            ("ls", []) => Self::Ls(String::from(".")),
            ("ls", [path]) => Self::Ls(path.to_string()),
            ("mkdir", [path]) => Self::MkDir(path.to_string()),
            ("touch", [path]) => Self::Touch(path.to_string()),
            ("write", [path]) => Self::Write(path.to_string()),
            ("read", [path]) => Self::Read(path.to_string()),
            ("rm", [path]) => Self::Rm(path.to_string()),
            ("lua", [path]) => Self::Lua(path.to_string()),
            _ => {
                return Err(
                    match HELP
                        .iter()
                        .find(|(usage, _)| usage.split(' ').next() == Some(name))
                    {
                        Some((usage, _)) => format!("usage: {usage}"),
                        None => format!("Unknown command: {name}"),
                    },
                )
            }
        };
        Ok(Some(command))
    }
}

pub struct Shell {
    kernel: Kernel,
}

impl Shell {
    pub fn new() -> Self {
        let mut kernel = Kernel::new(TOTAL_MEMORY);
        kernel.vfs.create_dir_all("/bin").expect("empty vfs");
        kernel.vfs.create_dir_all("/tmp").expect("empty vfs");
        kernel.vfs.create_file("/.env").expect("empty vfs");
        Self { kernel }
    }

    pub fn run(&mut self, input: &mut impl BufRead, out: &mut impl Write) -> io::Result<()> {
        loop {
            write!(out, "kernelino> ")?;
            out.flush()?;

            let mut line = String::new();
            if input.read_line(&mut line)? == 0 {
                writeln!(out)?;
                return Ok(());
            }

            let command = match Command::parse(&line) {
                Ok(Some(command)) => command,
                Ok(None) => continue,
                Err(err) => {
                    writeln!(out, "{err}")?;
                    continue;
                }
            };

            if command == Command::Exit {
                writeln!(out, "Goodbye!")?;
                return Ok(());
            }

            // A failing command must never take the whole system down.
            match panic::catch_unwind(AssertUnwindSafe(|| self.execute(command, input, out))) {
                Ok(Ok(())) => {}
                Ok(Err(err)) => {
                    let name = line.split_whitespace().next().unwrap_or_default();
                    writeln!(out, "{name}: {err}")?;
                }
                Err(_) => writeln!(out, "internal error: the command was aborted")?,
            }
        }
    }

    fn execute(
        &mut self,
        command: Command,
        input: &mut impl BufRead,
        out: &mut impl Write,
    ) -> CommandResult {
        let kernel = &mut self.kernel;

        match command {
            Command::Exit => {}
            Command::Help => {
                writeln!(out, "Available commands:")?;
                for (usage, description) in HELP {
                    writeln!(out, "  {usage:<14} {description}")?;
                }
            }
            Command::Clear => utils::clear_terminal(out)?,
            Command::Pwd => writeln!(out, "{}", kernel.vfs.cwd().display())?,
            Command::Top => top(kernel)?,
            Command::Cd(path) => kernel.vfs.change_dir(&path)?,
            Command::Ls(path) => {
                for (name, node) in kernel.vfs.list(&path)? {
                    match node {
                        Node::Directory(_) => writeln!(out, "{name}/")?,
                        Node::File(file) => writeln!(out, "{name} {}", file.size)?,
                    }
                }
            }
            Command::MkDir(path) => kernel.vfs.create_dir_all(&path)?,
            Command::Touch(path) => kernel.vfs.create_file(&path)?,
            Command::Write(path) => {
                // Fail before the user types anything, unless only the file is missing.
                match kernel.vfs.file(&path) {
                    Err(VfsError::NotFound(missing)) if missing == kernel.vfs.resolve(&path) => {}
                    result => {
                        result?;
                    }
                }
                if let Some(content) = editor::edit(input, out)? {
                    kernel.write_file(&path, content.as_bytes())?;
                    writeln!(out, "File saved successfully!")?;
                }
            }
            Command::Read(path) => {
                let content = kernel.read_file(&path)?;
                out.write_all(&content)?;
                if !content.is_empty() && !content.ends_with(b"\n") {
                    writeln!(out)?;
                }
            }
            Command::Rm(path) => kernel.remove(&path)?,
            Command::Lua(path) => {
                let source = String::from_utf8(kernel.read_file(&path)?)?;
                let name = kernel.vfs.resolve(&path).display().to_string();
                let mut process = LuaProcess::spawn(&name, &source, &LUA_LIMITS)?;
                loop {
                    let step = process.step();
                    out.write_all(&process.take_output())?;
                    out.flush()?;
                    match step {
                        Step::Running => {}
                        Step::Finished => break,
                        Step::Failed(err) => return Err(err.into()),
                    }
                }
            }
        }
        Ok(())
    }
}

/// Live process view, refreshed every second until ESC is pressed.
fn top(kernel: &Kernel) -> io::Result<()> {
    let mut out = io::stdout();
    enable_raw_mode()?;

    let result = (|| -> io::Result<()> {
        loop {
            utils::clear_terminal(&mut out)?;
            write!(out, "Press ESC to stop showing processes\r\n\r\n")?;
            write!(out, "{:>5}  {:>8}  NAME\r\n", "PID", "TIME")?;
            for process in kernel.vpm.processes() {
                let seconds = process.uptime().as_secs();
                write!(
                    out,
                    "{:>5}  {:>7}s  {}\r\n",
                    process.pid, seconds, process.name
                )?;
            }
            write!(
                out,
                "\r\nMemory: {} MB free of {} MB\r\n",
                kernel.vmm.free_memory() / MEGABYTE,
                kernel.vmm.total_memory() / MEGABYTE
            )?;
            out.flush()?;

            if event::poll(Duration::from_secs(1))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Esc {
                        return Ok(());
                    }
                }
            }
        }
    })();

    disable_raw_mode()?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(script: &str) -> String {
        let mut out = Vec::new();
        Shell::new().run(&mut script.as_bytes(), &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn parse_matches_whole_words() {
        assert_eq!(
            Command::parse("cdfoo"),
            Err("Unknown command: cdfoo".into())
        );
        assert_eq!(
            Command::parse("  ls   d "),
            Ok(Some(Command::Ls("d".into())))
        );
        assert_eq!(Command::parse("cd"), Ok(Some(Command::Cd("/".into()))));
        assert_eq!(Command::parse("rm"), Err("usage: rm <path>".into()));
        assert_eq!(Command::parse("   "), Ok(None));
    }

    #[test]
    fn starts_with_base_filesystem() {
        let out = run("ls /\n");
        assert!(out.contains("bin/") && out.contains("tmp/") && out.contains(".env 0"));
    }

    #[test]
    fn end_of_input_stops_the_shell() {
        assert!(run("pwd\n").ends_with("kernelino> \n"));
    }

    #[test]
    fn rm_removes_directories() {
        assert!(!run("mkdir foo\nrm foo\nls\n").contains("foo/"));
    }

    #[test]
    fn errors_are_reported_and_the_shell_continues() {
        let out = run("cd nowhere\npwd\n");
        assert!(out.contains("cd: /nowhere: no such file or directory"));
        assert!(out.contains("kernelino> /\n"));
    }

    #[test]
    fn write_creates_and_read_prints() {
        let out = run("write notes.txt\nhello\n\nworld\nwq\nread notes.txt\n");
        assert!(out.contains("hello\n\nworld\n"));
    }

    #[test]
    fn write_into_missing_directory_fails_before_editing() {
        let out = run("write nodir/f.txt\n");
        assert!(out.contains("write: /nodir: no such file or directory"));
        assert!(!out.contains("Welcome to the editor"));
    }

    #[test]
    fn lua_scripts_print_to_the_shell() {
        let out = run(
            "write t.lua\nlocal my_var = 'hi from ' .. _VERSION\nprint(my_var)\nwq\nlua t.lua\n",
        );
        assert!(out.contains("hi from Lua 5.4\n"));
    }

    #[test]
    fn lua_errors_are_reported_and_the_shell_continues() {
        let out = run("write e.lua\nerror('boom')\nwq\nlua e.lua\npwd\n");
        assert!(out.contains("lua: /e.lua:1: boom\n"));
        assert!(out.contains("kernelino> /\n"));
    }

    #[test]
    fn lua_scripts_are_found_by_relative_path() {
        let out = run("mkdir d\nwrite d/x.lua\nprint('ok')\nwq\ncd /tmp\nlua ../d/x.lua\n");
        assert!(out.contains("ok\n"));
    }

    #[test]
    fn parse_reports_usage_for_wrong_arguments() {
        assert_eq!(Command::parse("ls a b"), Err("usage: ls [path]".into()));
        assert_eq!(Command::parse("exit now"), Err("usage: exit".into()));
        assert_eq!(Command::parse("cd a b"), Err("usage: cd [path]".into()));
        assert_eq!(Command::parse("write"), Err("usage: write <path>".into()));
        assert_eq!(Command::parse("lua"), Err("usage: lua <path>".into()));
    }

    #[test]
    fn parse_covers_every_command() {
        let path = || String::from("p");
        let expected = [
            Command::Exit,
            Command::Help,
            Command::Clear,
            Command::Pwd,
            Command::Cd(path()),
            Command::Ls(path()),
            Command::MkDir(path()),
            Command::Touch(path()),
            Command::Write(path()),
            Command::Read(path()),
            Command::Rm(path()),
            Command::Top,
            Command::Lua(path()),
        ];
        assert_eq!(HELP.len(), expected.len());
        for ((usage, _), command) in HELP.iter().zip(expected) {
            let line = usage.replace("<path>", "p").replace("[path]", "p");
            assert_eq!(Command::parse(&line), Ok(Some(command)), "{usage}");
        }
    }

    #[test]
    fn help_lists_every_command() {
        let out = run("help\n");
        for (usage, _) in HELP {
            assert!(out.contains(usage), "{usage}");
        }
    }

    #[test]
    fn ls_prints_directories_and_file_sizes() {
        let out = run("mkdir d\nwrite f\nhi\nwq\nls\n");
        assert!(out.contains("d/\n"));
        assert!(out.contains("f 3\n"));
    }

    #[test]
    fn editor_quit_does_not_create_the_file() {
        let out = run("write f\nabc\nq\nls\n");
        assert!(out.contains("bin/\n"));
        assert!(!out.contains("\nf "));
    }

    #[test]
    fn exit_stops_before_the_next_commands() {
        let out = run("exit\npwd\n");
        let (_, after) = out.split_once("Goodbye!").unwrap();
        assert!(!after.contains("/\n"));
    }

    #[test]
    fn clear_writes_the_clear_sequence() {
        assert!(run("clear\n").contains("\x1b[2J"));
    }

    #[test]
    fn rm_of_the_working_directory_is_refused() {
        let out = run("mkdir a\ncd a\nrm /a\n");
        assert!(out.contains("rm: /a: is the current directory or one of its parents"));
    }

    #[test]
    fn lua_on_a_missing_file_fails() {
        let out = run("lua nope.lua\n");
        assert!(out.contains("lua: /nope.lua: no such file or directory"));
    }

    #[test]
    fn lua_syntax_errors_are_reported() {
        let out = run("write bad.lua\nx = = 1\nwq\nlua bad.lua\n");
        assert!(out.contains("lua: /bad.lua:1:"));
    }
}
