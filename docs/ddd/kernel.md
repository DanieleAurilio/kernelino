# Kernel

## Responsibility

The Kernel owns the subsystems and keeps them consistent. It holds the [File system](filesystem.md) (`vfs: Vfs`), [Memory](memory.md) (`vmm: Vmm`) and the process table of [Processes](processes.md) (`vpm: Vpm`) as public fields; there is no global state, and the [Shell](shell.md) owns the Kernel.

Its rule: the File system knows the tree, Memory knows the bytes, and the Kernel makes sure every Page a File holds is a live Page of Memory, freed when the File is rewritten or removed. Every operation that touches both goes through it.

## Code

- [`src/kernel.rs`](../../src/kernel.rs): `Kernel`, `KernelError`.

## Concepts

### Ownership of the subsystems

`Kernel::new(total_memory)` creates an empty `Vfs` (only `/`), a `Vmm` of `total_memory` bytes and a `Vpm` with the `shell` Process. The Kernel itself uses only the `Vfs` and the `Vmm`: it owns the `Vpm` without calling it.

### `read_file(path)`

Finds the File (`Vfs::file`) and reads its `size` bytes from its Pages (`Vmm::read_bytes`). A File with no Pages reads as empty.

### `write_file(path, bytes)`

Replaces the content of a File, creating it if it is missing:
1. If the File is missing, it is created empty (`Vfs::create_file`). Any other error (the path is a Directory, its parent Directory is missing or is a File) is returned before any memory is allocated.
2. The bytes are copied into new Pages (`Vmm::allocate_bytes`), all or nothing.
3. The File's Pages and size are replaced, then its old Pages are freed (`Vmm::deallocate`).

Invariant: on `OutOfMemory` the File keeps its old content and no Page leaks. A File created by this same call stays, empty.

### `remove(path)`

Detaches the Node from the tree (`Vfs::remove`), then frees every Page it holds (`Vmm::deallocate` of `Node::pages`), including the Pages of every File nested in a removed Directory.

## Errors

`KernelError` wraps the error of the subsystem that failed; its message is the message of that error.

| Variant | Meaning |
|---|---|
| `KernelError::Vfs(VfsError)` | the path is wrong for the operation: missing, a Directory where a File is needed, the Working directory… (see [File system](filesystem.md#errors)) |
| `KernelError::Vmm(VmmError)` | `OutOfMemory` while writing, or `InvalidAddress` if a File held a Page that is not mapped (see [Memory](memory.md#errors)) |

## Not in this context

- It is not the only way into the subsystems: the Shell calls the `Vfs` directly through `kernel.vfs` for the operations that need no memory (`cd`, `ls`, `mkdir`, `touch`, path resolution), reads `kernel.vpm` and `kernel.vmm` for `top`, and spawns Lua processes itself (see [Context map](context-map.md)).
- It does not spawn, schedule or kill processes: Lua processes are not in the process table yet (roadmap M4).
- It does not account the memory of Lua states in the `Vmm` (a known issue of [`docs/CONTEXT.md`](../CONTEXT.md)).
- It defines no path rule and no allocation rule: those belong to the File system and Memory.
