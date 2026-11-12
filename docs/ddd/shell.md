# Shell

## Responsibility

The Shell is the user-facing context: it reads one line at a time, turns it into a [Command](glossary.md), runs it against the [Kernel](kernel.md) and prints the result. It is the only context that talks to the user. It also owns the start-up of the system: `Shell::new` creates the Kernel with `TOTAL_MEMORY` (4 GiB) and the base tree `/bin`, `/tmp` and the empty file `/.env`.

A failing Command never stops the Shell: its error is printed and the next prompt (`kernelino> `) follows.

## Code

- [`src/shell.rs`](../../src/shell.rs): `Shell`, `Command`, the `HELP` table, `LUA_LIMITS`, `top`.
- [`src/editor.rs`](../../src/editor.rs): the Editor (`editor::edit`).
- [`src/utils.rs`](../../src/utils.rs): `clear_terminal`.
- [`src/main.rs`](../../src/main.rs): entry point, `Shell::new().run(stdin, stdout)` after clearing the terminal.

## Concepts

### Command

One line typed at the prompt, parsed by `Command::parse` into a `Command` variant: `Exit`, `Help`, `Clear`, `Pwd`, `Top`, `Cd`, `Ls`, `MkDir`, `Touch`, `Write`, `Read`, `Rm`, `Lua`.

The command line grammar:
- The line is split on whitespace; the first word is the command name, the others its arguments. A path therefore cannot contain spaces.
- An empty or blank line is no Command (`Ok(None)`): nothing is printed.
- Each command takes an exact number of arguments, the ones its usage in `HELP` shows: `<path>` is required, `[path]` optional. `cd` without a path goes to `/`, `ls` without a path lists `.`.
- A known name with the wrong arguments is refused with `usage: <usage>`, the usage taken from `HELP` (e.g. `usage: rm <path>`).
- An unknown name is refused with `Unknown command: <name>`. Names are whole words: `cdfoo` is unknown, not `cd foo`.

`HELP` is the single list of the commands, their usage and description: `help` prints it, and the tests check that it matches `Command::parse` entry by entry.

What each Command does:

| Command | Runs |
|---|---|
| `exit` | prints `Goodbye!` and stops the Shell |
| `help` | prints `HELP` |
| `clear` | `utils::clear_terminal` |
| `pwd` | prints the Working directory (`kernel.vfs.cwd()`) |
| `cd [path]` | `kernel.vfs.change_dir` |
| `ls [path]` | `kernel.vfs.list`: a Directory as `name/`, a File as `name size` |
| `mkdir <path>` | `kernel.vfs.create_dir_all` (`mkdir -p` semantics, see [File system](filesystem.md)) |
| `touch <path>` | `kernel.vfs.create_file`: fails if the path exists |
| `write <path>` | the Editor, then `Kernel::write_file` |
| `read <path>` | `Kernel::read_file`, adding a final newline to a non-empty content that lacks one |
| `rm <path>` | `Kernel::remove` |
| `top` | the live process view |
| `lua <path>` | runs a Lua process in the foreground (see below) |

End of input (Ctrl-D, or the end of a piped script) stops the Shell like `exit`, without `Goodbye!`.

### Editor

The minimal line editor of `write` (`editor::edit`). Every line typed is appended to the content, empty lines included. A line that is `wq` (surrounding whitespace ignored) saves: `edit` returns the content and `write` stores it with `Kernel::write_file`, then prints `File saved successfully!`. A line that is `q`, or the end of input, discards: nothing is written and a file that did not exist is not created.

`write` checks the path before opening the Editor: it fails at once unless the path is an existing File or only the File itself is missing (its parent Directory exists), so the user never types content that cannot be saved.

### Running a Lua process

`lua <path>` reads the File through `Kernel::read_file`, requires it to be UTF-8, and spawns a `LuaProcess` named after the resolved absolute path with `LUA_LIMITS` (64 MB of memory, 1 000 000 000 instructions). The Shell then acts as the scheduler of this one process, in the foreground: it calls `step` until the Step is not `Running`, and after every Time slice writes and flushes the output the process printed (`take_output`). See [Processes](processes.md).

### Error output

- Errors of a Command are printed as `<command>: <error>`, `<command>` being the first word of the line (e.g. `cd: /nowhere: no such file or directory`, `lua: /e.lua:1: boom`).
- Parse errors are printed as they are (`usage: …`, `Unknown command: …`).

### Panic isolation

Every Command runs under `panic::catch_unwind`: a panic aborts the Command, prints `internal error: the command was aborted`, and the Shell goes on with the next line.

### `top`

A live view of the process table (`kernel.vpm.processes()`: PID, uptime in seconds, name) and of the memory (`kernel.vmm.free_memory()` and `total_memory()`, in MB), refreshed every second until ESC is pressed. It switches the terminal to raw mode and writes to the real standard output, not to the Shell's output.

## Errors

The Shell has no error type of its own: a Command returns `Result<(), Box<dyn Error>>` (`CommandResult`) and the Shell prints whatever error comes back.

| Error | Meaning |
|---|---|
| `usage: <usage>` | a known command with the wrong arguments |
| `Unknown command: <name>` | a name that is not in `HELP` |
| `VfsError`, `KernelError` | a file operation failed (see [File system](filesystem.md), [Kernel](kernel.md)) |
| a UTF-8 error | `lua` on a File that is not valid UTF-8 |
| the message of a spawn error or of `Step::Failed` | `lua`: a syntax error, a runtime error, the memory limit or a Kill (see [Processes](processes.md)) |
| `internal error: the command was aborted` | the Command panicked |
| I/O errors | inside a Command they are printed like any other error; reading the next line, or writing the prompt or an error, failing is the only error that stops `Shell::run` |

## Not in this context

- It stores no file and no byte: the tree is in the [File system](filesystem.md), the bytes in [Memory](memory.md), kept consistent by the [Kernel](kernel.md).
- It does not define path semantics: every path is passed as typed and resolved by the File system.
- It does not sandbox Lua: the limits, the Time slice and the Kill belong to [Processes](processes.md); the Shell only steps the process and prints its output.
- No background processes, no `ps` or `kill` command, no interrupt key for a foreground script (roadmap M4).
