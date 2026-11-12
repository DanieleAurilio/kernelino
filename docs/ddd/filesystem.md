# File system

## Responsibility

The File system is the in-memory tree of Directories and Files, navigated by path, and the Working directory. It knows names, kinds and sizes; it never holds the bytes of a File, only the Pages of [Memory](memory.md) that hold them. It has no parent pointers: every operation resolves its path to an absolute path, then walks the tree from the root.

## Code

- [`src/vfs.rs`](../../src/vfs.rs): `Vfs`, `Node`, `File`, `Directory`, `VfsError`.

## Concepts

### Node

An entry of the tree (`Node`): either a `Directory` or a `File`. Its kind is fixed when it is created and never depends on its name: `.env` is a File, `my.dir` can be a Directory. `Node::pages` returns every Page a Node holds, the ones of nested Files included.

### Directory

A Node holding named entries (`entries`: name → Node, kept sorted by name, so `ls` lists them in name order). The root `/` is a Directory that always exists and cannot be removed.

### File

A Node holding a `size` in bytes and the `pages` (virtual addresses of Pages of [Memory](memory.md)) that hold its bytes. A new File is empty: size 0, no Pages. Only the [Kernel](kernel.md) changes its Pages and size, through `Vfs::file_mut`.

### Working directory

The absolute path relative paths are resolved from (`cwd`, printed by `pwd`). It starts at `/` and changes only through `change_dir`, which accepts only an existing Directory. It and its parents cannot be removed (`InUse`).

### Path resolution

`Vfs::resolve(path)` turns any path into a normalized absolute path, without looking at the tree:
- a path starting with `/` is absolute; any other path is relative to the Working directory;
- the path is split on `/`; empty names and `.` are skipped; `..` drops the last name;
- `..` at the root stays at the root (`cd ..` in `/` stays in `/`, `/../..` is `/`).

Every public operation of `Vfs` resolves its path first, and every `VfsError` carries the resolved absolute path.

### Operations and their rules

- `create_dir_all(path)`: `mkdir -p` semantics. It creates the Directory and every missing parent. It fails with `AlreadyExists` only when nothing was created (the whole path already existed as Directories), and with `NotADirectory` when a File is on the way.
- `create_file(path)`: creates an empty File. The parent must be an existing Directory; the name must be free (a File or a Directory with that name gives `AlreadyExists`); `/` gives `IsADirectory`.
- `remove(path)`: detaches the Node, a File or a whole Directory with its content, and returns it: the caller owns its Pages (the Kernel frees them). It is refused with `InUse` when the path is the Working directory or one of its parents, so `/` can never be removed.
- `list(path)`: the entries of a Directory, with their Node.
- `file(path)`, `file_mut(path)`: the File at the path.
- `change_dir(path)`: sets the Working directory.

## Errors

`VfsError`, each variant carrying the resolved absolute path it is about:

| Variant | Message | Meaning |
|---|---|---|
| `NotFound` | `<path>: no such file or directory` | the path, or the parent Directory it needs, does not exist; the path carried is the one that was looked up (e.g. `write x/f` with no `/x` reports `/x`) |
| `AlreadyExists` | `<path>: already exists` | `create_file` on a name in use, `create_dir_all` on an existing Directory |
| `NotADirectory` | `<path>: not a directory` | a File where a Directory is needed: `cd`, `ls`, a parent in the path, `mkdir` through a File |
| `IsADirectory` | `<path>: is a directory` | a Directory where a File is needed: `read`, `write`, `file`, `file_mut`, or `/` as a File |
| `InUse` | `<path>: is the current directory or one of its parents` | `remove` of the Working directory or of one of its parents |

## Not in this context

- No bytes: File contents live in [Memory](memory.md), and the [Kernel](kernel.md) keeps a File's Pages valid.
- No parent pointers, no links, no rename or move.
- No users nor permissions (roadmap M5).
- Lua scripts cannot reach it: mapping Lua `io` to the File system is roadmap M4.
