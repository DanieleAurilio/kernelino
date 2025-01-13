/**
 * Virtual Memory Manager
 */

use std::collections::HashMap;

const DEFAULT_PAGE_SIZE: u64 = 4096;

#[derive(Debug, Clone)]
struct Frame {
    id: u64,
    address: u64,
    in_use: bool
}

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
#[derive(Debug, Clone)]
pub struct PageTableEntry {
    virtual_address: u64,
    physical_address: u64,
    flags: u8
}

#[derive(Debug, Clone)]
pub struct Vmm {
    pub total_memory: u64,
    pub free_memory: u64,
    frames: Vec<Frame>,
    page_table: HashMap<u64, PageTableEntry>
}

impl Vmm {
    pub fn new(total_memory: u64) -> Self {
        let num_frames = total_memory / DEFAULT_PAGE_SIZE;
        let mut frames = Vec::new();
        for frame_id in 0..num_frames {
            frames.push(Frame {
                id: frame_id,
                in_use: false,
                address: frame_id * DEFAULT_PAGE_SIZE
            });
        }

        Self {
            free_memory: total_memory,
            page_table: HashMap::new(),
            total_memory,
            frames
        }
    }

    pub fn allocate_page(&mut self) {
        let frame = self.frames.pop().expect("Out of memory allocating page.");
        let virtual_address = frame.address + DEFAULT_PAGE_SIZE;
        let page = PageTableEntry {
            physical_address: frame.address,
            virtual_address,
            flags: 0b0000_0111,
        };
        self.page_table.insert(virtual_address, page);
    }

    pub fn deallocate_page(&mut self, virtual_address: u64) {
        match self.page_table.remove(&virtual_address) {
            Some(_p) => (),
            None => {
                println!("Cannot deallocate {}", virtual_address);
            }
        }
    }
}