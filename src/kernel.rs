/**
 * Kernel
 *
 * Owns the subsystems and coordinates them: the VFS knows the tree,
 * the VMM knows the bytes, the kernel keeps them consistent.
 */
use std::fmt;

use crate::vfs::{Node, Vfs, VfsError};
use crate::vmm::{Vmm, VmmError};
use crate::vpm::Vpm;

#[derive(Debug, PartialEq, Eq)]
pub enum KernelError {
    Vfs(VfsError),
    Vmm(VmmError),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vfs(err) => err.fmt(f),
            Self::Vmm(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for KernelError {}

impl From<VfsError> for KernelError {
    fn from(err: VfsError) -> Self {
        Self::Vfs(err)
    }
}

impl From<VmmError> for KernelError {
    fn from(err: VmmError) -> Self {
        Self::Vmm(err)
    }
}

pub struct Kernel {
    pub vfs: Vfs,
    pub vmm: Vmm,
    pub vpm: Vpm,
}

impl Kernel {
    pub fn new(total_memory: u64) -> Self {
        Self {
            vfs: Vfs::new(),
            vmm: Vmm::new(total_memory),
            vpm: Vpm::new(),
        }
    }

    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, KernelError> {
        let file = self.vfs.file(path)?;
        Ok(self.vmm.read_bytes(&file.pages, file.size)?)
    }

    /// Replaces the file content, creating the file if it does not exist.
    /// If memory runs out the old content is kept.
    pub fn write_file(&mut self, path: &str, bytes: &[u8]) -> Result<(), KernelError> {
        match self.vfs.file(path) {
            Err(VfsError::NotFound(_)) => self.vfs.create_file(path)?,
            result => {
                result?;
            }
        }

        let pages = self.vmm.allocate_bytes(bytes)?;
        let file = self.vfs.file_mut(path)?;
        let old_pages = std::mem::replace(&mut file.pages, pages);
        file.size = bytes.len() as u64;
        self.vmm.deallocate(&old_pages)?;
        Ok(())
    }

    /// Removes a file or a whole directory and frees its memory.
    pub fn remove(&mut self, path: &str) -> Result<(), KernelError> {
        let node: Node = self.vfs.remove(path)?;
        self.vmm.deallocate(&node.pages())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vmm::PAGE_SIZE;

    fn kernel() -> Kernel {
        Kernel::new(PAGE_SIZE * 16)
    }

    #[test]
    fn removing_a_file_keeps_other_files_intact() {
        let mut kernel = kernel();
        kernel.vfs.create_file("a.txt").unwrap();
        kernel.vfs.create_file("b.txt").unwrap();
        kernel.write_file("b.txt", b"hello").unwrap();
        kernel.remove("a.txt").unwrap();
        assert_eq!(kernel.read_file("b.txt").unwrap(), b"hello");
    }

    #[test]
    fn rewriting_frees_the_old_pages() {
        let mut kernel = kernel();
        kernel
            .write_file("f", &[1; PAGE_SIZE as usize * 3])
            .unwrap();
        kernel.write_file("f", b"small").unwrap();
        assert_eq!(kernel.vmm.free_memory(), PAGE_SIZE * 15);
        assert_eq!(kernel.read_file("f").unwrap(), b"small");
    }

    #[test]
    fn files_larger_than_a_page_are_read_whole() {
        let mut kernel = kernel();
        let data = vec![b'x'; PAGE_SIZE as usize * 2 + 1];
        kernel.write_file("big", &data).unwrap();
        assert_eq!(kernel.read_file("big").unwrap(), data);
    }

    #[test]
    fn empty_files_can_be_read() {
        let mut kernel = kernel();
        kernel.vfs.create_file("e").unwrap();
        assert!(kernel.read_file("e").unwrap().is_empty());
        kernel.write_file("e", b"").unwrap();
        assert!(kernel.read_file("e").unwrap().is_empty());
    }

    #[test]
    fn out_of_memory_keeps_old_content() {
        let mut kernel = Kernel::new(PAGE_SIZE);
        kernel.write_file("f", b"old").unwrap();
        let too_big = vec![0; PAGE_SIZE as usize * 2];
        assert_eq!(
            kernel.write_file("f", &too_big),
            Err(KernelError::Vmm(VmmError::OutOfMemory))
        );
        assert_eq!(kernel.read_file("f").unwrap(), b"old");
    }

    #[test]
    fn removing_a_directory_frees_nested_files() {
        let mut kernel = kernel();
        kernel.vfs.create_dir_all("d/e").unwrap();
        kernel.write_file("d/e/f", b"data").unwrap();
        kernel.remove("d").unwrap();
        assert_eq!(kernel.vmm.free_memory(), PAGE_SIZE * 16);
    }
}
