# Glossary

The terms of kernelino's domain. Use them with these meanings in code, docs and commits. Each term belongs to one bounded context, whose page gives its invariants.

| Term | Context | Definition |
|---|---|---|
| Command | [Shell](shell.md) | One line typed at the prompt, parsed by `Command::parse` into a `Command` (`exit`, `cd`, `write`, `lua`…). Its name and arguments must match a usage of the `HELP` table, else it is refused with `usage: …` or `Unknown command: …`; its errors are printed as `<command>: <error>` and the Shell goes on. |
| Editor | [Shell](shell.md) | The minimal line editor of `write` (`editor::edit`): every line typed is appended to the content; `wq` saves it, `q` or the end of input discards it. |
| Node | [File system](filesystem.md) | An entry of the tree, either a Directory (named entries) or a File (a size and the Pages holding its bytes). Its kind never depends on its name: `.env` is a File, `my.dir` can be a Directory. |
| Directory | [File system](filesystem.md) | A Node holding named entries (other Nodes), sorted by name. The root `/` is a Directory and cannot be removed. |
| File | [File system](filesystem.md) | A Node holding a size in bytes and the Pages of Memory that hold its bytes; it never holds the bytes itself. A new File is empty: size 0, no Pages. |
| Working directory | [File system](filesystem.md) | The absolute path relative paths are resolved from (`cwd`, printed by `pwd`). It and its parents cannot be removed. |
| Page / Frame | [Memory](memory.md) | A Frame is a unit of simulated physical memory of PAGE_SIZE (4096) bytes; a Page is the virtual address a Frame is mapped to. Files hold Pages, never Frames. A freed Frame is reused by the next allocation. |
| Page table | [Memory](memory.md) | The map from every live Page to its Frame and flags (`PageTableEntry`). A Page is valid exactly while it is in the Page table; the used memory is its number of entries times PAGE_SIZE. |
| Process | [Processes](processes.md) | An entry of the process table (`Vpm`): a PID, a name and an uptime. Today the table holds only `shell`, PID 1. |
| Lua process | [Processes](processes.md) | A Lua 5.4 script run as a process (`lua <path>`), in its own Lua state, with only safe libraries and no host access (no io, os, require, debug, bytecode). Its `print` goes to a per-process output the Shell drains after every Time slice. It ends Finished or Failed. |
| Limits | [Processes](processes.md) | The resources a Lua process may use: a memory limit for its Lua state and a total instruction budget (CPU limit). Exceeding either fails the process; the Shell keeps running. |
| Time slice / Preemption | [Processes](processes.md) | A Time slice is the number of instructions a Lua process runs before control returns to the scheduler; Preemption is that forced return, invisible to the script even inside its own coroutines. |
| Kill | [Processes](processes.md) | The termination of a Lua process by the system, on CPU limit or when stuck past its Time slice where it cannot be preempted. A Kill cannot be caught (pcall, xpcall, coroutine.resume). |
| Step | [Processes](processes.md) | The outcome of running a Lua process for one Time slice (`LuaProcess::step`): `Running` (it can run again), `Finished` (the script returned) or `Failed` (an uncaught error, the memory limit or a Kill, with its message). |
