![kernelino_resized](https://github.com/user-attachments/assets/32e2163c-46e3-494d-b6e0-5ad5b13a8dcd)

# kernelino

kernelino is a sandbox OS shell written in Rust that runs real Lua 5.4 scripts as processes, confined to RAM: a script (even malicious) must never crash or hang the system, reach the host filesystem or the network. Files, memory and processes are all virtual and live in memory.

## Run it

```sh
cargo run
```

A C compiler is needed: Lua is vendored and compiled by the build.

## Commands

| Command | Description |
|---------|-------------|
| `exit` | Exit the shell |
| `help` | Display this help message |
| `clear` | Clear the screen |
| `pwd` | Print the current working directory |
| `cd [path]` | Change the current working directory |
| `ls [path]` | List directory contents |
| `mkdir <path>` | Create a directory and its missing parents |
| `touch <path>` | Create an empty file |
| `write <path>` | Write a file with the editor, creating it if needed |
| `read <path>` | Print file content |
| `rm <path>` | Remove a file, or a directory with its content |
| `top` | Show the processes |
| `lua <path>` | Run a Lua 5.4 script in the sandbox |

## Documentation

The architecture, the decisions and the domain model are documented in [`docs/README.md`](docs/README.md).
