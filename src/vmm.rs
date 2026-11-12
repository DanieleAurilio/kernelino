/**
 * Virtual Memory Manager
 *
 * Maps virtual pages to physical frames of PAGE_SIZE bytes.
 * Frames are handed out lazily from a free list, so the simulated
 * memory size does not cost real memory until it is used.
 */
use std::{cmp::min, collections::HashMap, fmt};

pub const PAGE_SIZE: u64 = 4096;

/**
 * flags assume binaries values.
 *
 * P (Present): 0b0000_0001
 * R/W (Read/Write): 0b0000_0010
 * U/S (User/Supervisor): 0b0000_0100
 * PCD (Page-Level Cache Disable): 0b0000_1000
 * A (Accessed): 0b0001_0000
 * D (Dirty): 0b0010_0000
 * PS (Page Size): 0b0100_0000
 * PG (Page Global): 0b1000_0000
 *
 */
const DEFAULT_PAGE_FLAGS: u8 = 0b0000_0111;

#[derive(Debug, PartialEq, Eq)]
pub enum VmmError {
    OutOfMemory,
    InvalidAddress(u64),
}

impl fmt::Display for VmmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory => write!(f, "out of memory"),
            Self::InvalidAddress(address) => write!(f, "invalid virtual address {address}"),
        }
    }
}

impl std::error::Error for VmmError {}

#[derive(Debug)]
#[allow(dead_code)]
struct PageTableEntry {
    frame: u64,
    flags: u8,
}

#[derive(Debug)]
pub struct Vmm {
    frame_count: u64,
    next_unused_frame: u64,
    free_frames: Vec<u64>,
    frames: HashMap<u64, Vec<u8>>,
    page_table: HashMap<u64, PageTableEntry>,
    next_virtual_address: u64,
}

impl Vmm {
    pub fn new(total_memory: u64) -> Self {
        Self {
            frame_count: total_memory / PAGE_SIZE,
            next_unused_frame: 0,
            free_frames: Vec::new(),
            frames: HashMap::new(),
            page_table: HashMap::new(),
            next_virtual_address: 0,
        }
    }

    pub fn total_memory(&self) -> u64 {
        self.frame_count * PAGE_SIZE
    }

    pub fn free_memory(&self) -> u64 {
        (self.frame_count - self.page_table.len() as u64) * PAGE_SIZE
    }

    pub fn allocate_page(&mut self) -> Result<u64, VmmError> {
        let frame = match self.free_frames.pop() {
            Some(frame) => frame,
            None if self.next_unused_frame < self.frame_count => {
                self.next_unused_frame += 1;
                self.next_unused_frame - 1
            }
            None => return Err(VmmError::OutOfMemory),
        };

        let virtual_address = self.next_virtual_address;
        self.next_virtual_address += 1;
        self.frames.insert(frame, Vec::new());
        self.page_table.insert(
            virtual_address,
            PageTableEntry {
                frame,
                flags: DEFAULT_PAGE_FLAGS,
            },
        );

        Ok(virtual_address)
    }

    /// Frees every page, or none of them if any address is invalid.
    pub fn deallocate(&mut self, virtual_addresses: &[u64]) -> Result<(), VmmError> {
        if let Some(&address) = virtual_addresses
            .iter()
            .find(|address| !self.page_table.contains_key(address))
        {
            return Err(VmmError::InvalidAddress(address));
        }

        for address in virtual_addresses {
            if let Some(page) = self.page_table.remove(address) {
                self.frames.remove(&page.frame);
                self.free_frames.push(page.frame);
            }
        }
        Ok(())
    }

    /// Copies `bytes` into newly allocated pages. On failure nothing stays allocated.
    pub fn allocate_bytes(&mut self, bytes: &[u8]) -> Result<Vec<u64>, VmmError> {
        let mut virtual_addresses = Vec::new();

        for chunk in bytes.chunks(PAGE_SIZE as usize) {
            let virtual_address = match self.allocate_page() {
                Ok(address) => address,
                Err(err) => {
                    self.deallocate(&virtual_addresses)?;
                    return Err(err);
                }
            };
            let frame = self.page_table[&virtual_address].frame;
            self.frames.insert(frame, chunk.to_vec());
            virtual_addresses.push(virtual_address);
        }

        Ok(virtual_addresses)
    }

    pub fn read_bytes(&self, virtual_addresses: &[u64], size: u64) -> Result<Vec<u8>, VmmError> {
        let mut bytes = Vec::with_capacity(size as usize);

        for &address in virtual_addresses {
            let page = self
                .page_table
                .get(&address)
                .ok_or(VmmError::InvalidAddress(address))?;
            let content = &self.frames[&page.frame];
            let remaining = size as usize - bytes.len();
            bytes.extend_from_slice(&content[..min(remaining, content.len())]);
        }

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_get_distinct_frames() {
        let mut vmm = Vmm::new(PAGE_SIZE * 2);
        let a = vmm.allocate_page().unwrap();
        let b = vmm.allocate_page().unwrap();
        assert_ne!(vmm.page_table[&a].frame, vmm.page_table[&b].frame);
        assert_eq!(vmm.allocate_page(), Err(VmmError::OutOfMemory));
    }

    #[test]
    fn deallocated_frames_are_reused() {
        let mut vmm = Vmm::new(PAGE_SIZE);
        let a = vmm.allocate_page().unwrap();
        vmm.deallocate(&[a]).unwrap();
        assert_eq!(vmm.free_memory(), PAGE_SIZE);
        assert!(vmm.allocate_page().is_ok());
    }

    #[test]
    fn deallocate_invalid_address_changes_nothing() {
        let mut vmm = Vmm::new(PAGE_SIZE * 2);
        let a = vmm.allocate_page().unwrap();
        assert_eq!(vmm.deallocate(&[a, 99]), Err(VmmError::InvalidAddress(99)));
        assert_eq!(vmm.free_memory(), PAGE_SIZE);
    }

    #[test]
    fn bytes_round_trip_across_pages() {
        let mut vmm = Vmm::new(PAGE_SIZE * 4);
        let data: Vec<u8> = (0..PAGE_SIZE * 2 + 10).map(|i| i as u8).collect();
        let pages = vmm.allocate_bytes(&data).unwrap();
        assert_eq!(pages.len(), 3);
        assert_eq!(vmm.read_bytes(&pages, data.len() as u64).unwrap(), data);
    }

    #[test]
    fn empty_bytes_use_no_pages() {
        let mut vmm = Vmm::new(PAGE_SIZE);
        let pages = vmm.allocate_bytes(&[]).unwrap();
        assert!(pages.is_empty());
        assert!(vmm.read_bytes(&pages, 0).unwrap().is_empty());
    }

    #[test]
    fn out_of_memory_releases_partial_allocation() {
        let mut vmm = Vmm::new(PAGE_SIZE);
        let data = vec![0u8; PAGE_SIZE as usize + 1];
        assert_eq!(vmm.allocate_bytes(&data), Err(VmmError::OutOfMemory));
        assert_eq!(vmm.free_memory(), PAGE_SIZE);
    }

    #[test]
    fn errors_display() {
        assert_eq!(VmmError::OutOfMemory.to_string(), "out of memory");
        assert_eq!(
            VmmError::InvalidAddress(7).to_string(),
            "invalid virtual address 7"
        );
    }

    #[test]
    fn reading_an_invalid_address_fails() {
        let vmm = Vmm::new(PAGE_SIZE);
        assert_eq!(vmm.read_bytes(&[42], 1), Err(VmmError::InvalidAddress(42)));
    }

    #[test]
    fn total_memory_is_rounded_down_to_pages() {
        assert_eq!(Vmm::new(PAGE_SIZE * 2 + 1).total_memory(), PAGE_SIZE * 2);
    }

    #[test]
    fn free_memory_decreases_with_allocations() {
        let mut vmm = Vmm::new(PAGE_SIZE * 4);
        vmm.allocate_page().unwrap();
        vmm.allocate_page().unwrap();
        assert_eq!(vmm.free_memory(), PAGE_SIZE * 2);
    }
}
