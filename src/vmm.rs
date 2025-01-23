/**
 * Virtual Memory Manager
 */

use std::{cmp::min, collections::HashMap};

const DEFAULT_PAGE_SIZE: u64 = 4096;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Frame {
    id: u64,
    address: u64,
    in_use: bool,
    content: Option<Vec<u8>>
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
#[allow(dead_code)]
pub struct PageTableEntry {
    virtual_address: u64,
    physical_address: u64,
    flags: u8
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Vmm {
    pub total_memory: u64,
    pub free_memory: u64,
    frames: Vec<Frame>,
    page_table: HashMap<u64, PageTableEntry>,
    next_virtual_address: u64
}

impl Vmm {
    pub fn new(total_memory: u64) -> Self {
        let num_frames = total_memory / DEFAULT_PAGE_SIZE;
        let mut frames = Vec::new();
        for frame_id in 0..num_frames {
            frames.push(Frame {
                id: frame_id,
                in_use: false,
                address: frame_id * DEFAULT_PAGE_SIZE,
                content: None
            });
        }

        Self {
            free_memory: total_memory,
            page_table: HashMap::new(),
            total_memory,
            frames,
            next_virtual_address: 0
        }
    }

    pub fn allocate_page(&mut self) -> (u64, u64)  {
        let frame = self.frames.iter().find(|f| !f.in_use).expect("Out of memory allocating page.");
        let virtual_address = self.next_virtual_address;
        self.next_virtual_address += 1;

        let page = PageTableEntry {
            physical_address: frame.address,
            virtual_address,
            flags: 0b0000_0111,
        };
        self.page_table.insert(virtual_address, page);
        self.free_memory -= DEFAULT_PAGE_SIZE;

       (virtual_address, DEFAULT_PAGE_SIZE)
    }

    pub fn deallocate_page(&mut self, virtual_addresses: Vec<u64>) {    
        virtual_addresses.iter().for_each(|address| {
            if let Some(page) = self.page_table.remove(&address) {
                self.free_memory += DEFAULT_PAGE_SIZE;
                if let Some(frame) = self.frames.iter_mut().find(|f| f.address == page.physical_address) {
                    frame.in_use = false;
                    frame.content = None;
                }
            } else {
                panic!("Cannot deallocate page {}", address);
            }
        });
    }

    pub fn allocate_bytes(&mut self, bytes: Vec<u8>) -> Vec<u64> {
        let mut remaining_bytes = bytes.as_slice();
        let mut virtual_addresses: Vec<u64> = Vec::<u64>::new();

        while !remaining_bytes.is_empty() {
            let (virtual_address, _) = self.allocate_page();
            let page = self.page_table.get_mut(&virtual_address).unwrap();
            if let Some(frame) = self.frames.iter_mut().find(|f|f.address == page.physical_address) {
                frame.in_use = true;

                let bytes_to_copy = min(remaining_bytes.len(), DEFAULT_PAGE_SIZE as usize);
                frame.content = Some(remaining_bytes[..bytes_to_copy].to_vec());
                remaining_bytes = &remaining_bytes[bytes_to_copy..];
            }

            virtual_addresses.push(virtual_address);
        }

        virtual_addresses
    }

    pub fn get_bytes(&self, virtual_addresses: Vec<u64>, size: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut remaining_size = size;
        let mut current_virtual_address = virtual_addresses[0];
        virtual_addresses.iter().for_each(|&address| {
            let page = self.page_table.get(&address).expect("Page not found");
            let frame = self.frames.iter().find(|f| f.address == page.physical_address).expect("Frame not found");
            let content = frame.content.as_ref().expect("Content not found");

            let bytes_to_copy = min(remaining_size as usize, content.len());
            bytes.extend_from_slice(&content[..bytes_to_copy]);
            remaining_size -= bytes_to_copy as u64;
            current_virtual_address += 1;
        });

        bytes
    }
}