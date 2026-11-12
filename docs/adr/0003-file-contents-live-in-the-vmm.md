# 0003. File contents live in the VMM

- Status: accepted
- Date: 2026-11-12

## Context

kernelino simulates memory with a virtual memory manager (`src/vmm.rs`): a page table maps virtual pages of `PAGE_SIZE` (4096) bytes to frames, which are allocated lazily. Files hold bytes, so they need memory too.

The 2026-04-02 code review found that the previous design read a file as a fixed 4096 bytes whatever its size: "For files smaller than a page this reads garbage padding; for files spanning multiple pages it truncates."

## Decision

A `File` (`src/vfs.rs`) holds only its `size` and the virtual addresses of its `pages`. The bytes live in `Vmm` frames.

`Kernel` (`src/kernel.rs`) is the only place that touches both subsystems for a file:

- `Kernel::write_file` allocates the new pages with `Vmm::allocate_bytes`, swaps them into the `File`, sets its `size`, then frees the old pages with `Vmm::deallocate`.
- `Kernel::read_file` reads exactly `size` bytes from the file's pages with `Vmm::read_bytes`.
- `Kernel::remove` detaches the node with `Vfs::remove` and frees every page under it (`Node::pages`).

The VFS on its own never allocates or frees memory.

## Alternatives considered

- **Bytes inside the VFS node.** Rejected: the VMM would then simulate nothing, and file contents would not count against the simulated memory.
- **A fixed 4096-byte read regardless of the file size** (the previous design). Rejected: the 2026-04-02 code review flagged it as returning padding for small files and truncating large ones. Reads now use `size` and every page of the file.

## Consequences

- The new pages are allocated before the old ones are freed, so a rewrite that runs out of memory fails with `VmmError::OutOfMemory` and keeps the old content.
- Removing a directory frees the pages of every nested file.
- Files of any size, including empty ones and ones spanning several pages, are read whole.
- Writing or removing a file must go through `Kernel`: calling `Vfs::remove` directly would leak the file's pages.
