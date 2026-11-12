# Memory

## Responsibility

Memory simulates the physical memory of kernelino and maps virtual Pages to physical Frames. It stores the bytes of every File of the [File system](filesystem.md), and knows nothing about files or paths: it hands out Pages, stores and returns bytes, and frees Pages when asked.

## Code

- [`src/vmm.rs`](../../src/vmm.rs): `Vmm`, `PageTableEntry`, `PAGE_SIZE`, `DEFAULT_PAGE_FLAGS`, `VmmError`.

## Concepts

### Frame

A unit of simulated physical memory of `PAGE_SIZE` (4096) bytes, numbered from 0 to the frame count (`total_memory / PAGE_SIZE`, rounded down; the Shell asks for 4 GiB). Frames are lazy: a Frame costs real memory only while a Page is mapped to it, and holds only the bytes actually written to it (at most `PAGE_SIZE`). Invariant: a Frame is mapped by at most one Page.

### Page

The virtual address a Frame is mapped to. Files hold Pages, never Frames. A Page's address is a page number, not a byte address: each allocation takes the next one, and an address is never handed out twice, even after it is freed.

### Page table

The map from every live Page to its Frame and flags (`page_table`, entries `PageTableEntry { frame, flags }`). A Page is valid exactly while it is in the Page table. The used memory is the number of Pages in it times `PAGE_SIZE`: `free_memory` is `total_memory` minus that.

### Free list

Freed Frames go to a free list (`free_frames`), and the next allocation reuses the last one freed before touching a Frame never used (`next_unused_frame`). When neither is left, the allocation fails with `OutOfMemory`.

### Allocation and deallocation

- `allocate_page`: maps one new Page to a Frame.
- `allocate_bytes(bytes)`: splits the bytes in chunks of `PAGE_SIZE` and copies each into a new Page; empty bytes take no Page. All or nothing: if memory runs out, the Pages already allocated by this call are freed before `OutOfMemory` is returned.
- `deallocate(pages)`: frees every Page and returns its Frame to the free list. All or nothing: every address is checked first, and if one is not in the Page table, nothing is freed and `InvalidAddress` names it.
- `read_bytes(pages, size)`: concatenates the bytes of the Pages, up to `size` bytes.

### Page flags

Every Page table entry stores `DEFAULT_PAGE_FLAGS` (`0b0000_0111`: present, read/write, user), but the flags are never read: no protection is enforced.

## Errors

`VmmError`:

| Variant | Message | Meaning |
|---|---|---|
| `OutOfMemory` | `out of memory` | no Frame is left for the allocation; nothing stays allocated |
| `InvalidAddress(address)` | `invalid virtual address <address>` | a Page that is not in the Page table was read or freed; nothing changed |

## Not in this context

- No files nor paths: which Pages belong to which File is the business of the [File system](filesystem.md) and the [Kernel](kernel.md).
- No per-process address space, no swapping, no protection from the Page flags.
- It does not account the memory of Lua states: their limit is enforced by Lua itself (see [Processes](processes.md)); accounting it here is a known issue of [`docs/CONTEXT.md`](../CONTEXT.md).
