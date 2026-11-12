# 0002. VFS without parent pointers

- Status: accepted
- Date: 2026-11-12

## Context

The virtual file system (`src/vfs.rs`) is an in-memory tree of directories and files. Commands such as `cd ..`, `ls ../x` or `rm d` need to move upwards in the tree as well as downwards.

A tree whose nodes point to their parent has cycles. In Rust that means `Rc<RefCell<…>>` with `Weak` back-references, or `unsafe` raw pointers. Roadmap milestone M1 in `docs/CONTEXT.md` removed the VFS `parent` field.

## Decision

A `Directory` stores only its entries (`BTreeMap<String, Node>`); no node knows its parent. `Vfs` keeps the root and the working directory as a path (`cwd: PathBuf`).

Every path given to the VFS is first turned into a normalized absolute path by `Vfs::resolve`, which handles `.`, `..` and relative paths against the working directory. That path is then walked from the root by `Vfs::dir` or `Vfs::dir_mut`. Operations that work on a last name, such as `Vfs::create_file`, `Vfs::file` or `Vfs::remove`, split the path into its parent and last name and walk to the parent.

## Alternatives considered

- **Parent pointers in each node.** Rejected: the cycles force `Rc<RefCell>` (borrows checked at run time, so they can panic, and more code to keep the pointers consistent on every insert and remove) or `unsafe`. Owning the tree directly keeps it a plain value with no shared ownership.

## Consequences

- Every lookup walks from the root, which costs O(depth). That is fine for an in-memory shell.
- `..` is resolved on the path, not on the tree, so `..` from the root stays at the root.
- Removing a node detaches it with its whole subtree (`Vfs::remove` returns the `Node`). Removing the working directory or one of its parents would leave `cwd` pointing nowhere, so `Vfs::remove` refuses it with `VfsError::InUse`.
- The tree has a single owner (`Vfs`), in line with [ADR 0001](0001-tree-ownership-without-global-state.md).
