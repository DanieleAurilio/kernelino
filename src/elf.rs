use std::io::{Cursor, Write};
use byteordered::{self, byteorder::LittleEndian, byteorder::WriteBytesExt};

/**
 * This file contains the ELF struct and its implementation.
 * Kernelino uses the ELF format to load and execute binaries.
 * ELF is loaded in memory and executed by the VMM.
 * 
 * https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
 */

pub struct ELF {
    header: ELFHeader,
    program_header: ELFProgramHeader,
    sections: ELFSection,
}

struct ELFHeader {
    magic_number: u32,
    class: u8,
    data: u8,
    version: u8,
    os_abi: u8,
    abi_version: u8,
    padding: [u8; 7],
    file_type: u16,
    machine: u16,
    version_elf: u8,
    entry_point: u64,
    program_header_offset: u64,
    section_header_offset: u64,
    flags: u32,
    header_size: u16,
    program_header_size: u16,
    program_header_num: u16,
    section_header_size: u16,
    section_header_num: u16,
    section_header_str_index: u16,
}

struct ELFProgramHeader {
    program_header_type: u32,
    program_header_flags: u32,
    program_header_offset: u64,
    program_header_vaddr: u64,
    program_header_paddr: u64,
    program_header_filesz: u64,
    program_header_memsz: u64,
    program_header_align: u64,
}

struct ELFSection {
    body: Vec<u8>,
}

impl ELF { 
    pub fn new(bytes: Vec<u8>) -> Self {
        let elf_header: ELFHeader = ELFHeader {
            magic_number: 0x7F454C46,
            class: 0x04,
            data: 0x10,
            version: 0x01,
            os_abi: 0x00,
            abi_version: 0x00,
            padding: [0; 7],
            file_type: 0x02,
            machine: 0x00,
            version_elf: 0x01,
            entry_point: 0x00,
            program_header_offset: 0x40,
            section_header_offset: 0x00,
            flags: 0x00,
            header_size: 0x34,
            program_header_size: 0x38,
            program_header_num: 0x01,
            section_header_size: 0x40,
            section_header_num: 0x00,
            section_header_str_index: 0x00,
        };

        let elf_program_header: ELFProgramHeader = ELFProgramHeader {
            program_header_type: 0x01,
            program_header_flags: 0x05,
            program_header_offset: 0x00,
            program_header_vaddr: 0x400000,
            program_header_paddr: 0x400000,
            program_header_filesz: bytes.len() as u64,
            program_header_memsz: bytes.len() as u64,
            program_header_align: 0x200000,
        };

        let elf_section: ELFSection = ELFSection {
            body: bytes,
        };

        Self {
            header: elf_header,
            program_header: elf_program_header,
            sections: elf_section,
        }
    }

    #[allow(unused)]
    pub async fn to_bytes(&self) -> Vec<u8> {
        let mut elf_bytes: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut elf_bytes);

        // Write ELF Header
        cursor.write_u32::<LittleEndian>(self.header.magic_number);
        cursor.write_u8(self.header.class);
        cursor.write_u8(self.header.data);
        cursor.write_u8(self.header.version);
        cursor.write_u8(self.header.os_abi);
        cursor.write_u8(self.header.abi_version);
        self.header.padding.iter().for_each(|&x| { cursor.write_u8(x); });
        cursor.write_u16::<LittleEndian>(self.header.file_type);
        cursor.write_u16::<LittleEndian>(self.header.machine);
        cursor.write_u8(self.header.version_elf);
        cursor.write_u64::<LittleEndian>(self.header.entry_point);
        cursor.write_u64::<LittleEndian>(self.header.program_header_offset);
        cursor.write_u64::<LittleEndian>(self.header.section_header_offset);
        cursor.write_u32::<LittleEndian>(self.header.flags);
        cursor.write_u16::<LittleEndian>(self.header.header_size);
        cursor.write_u16::<LittleEndian>(self.header.program_header_size);
        cursor.write_u16::<LittleEndian>(self.header.program_header_num);
        cursor.write_u16::<LittleEndian>(self.header.section_header_size);
        cursor.write_u16::<LittleEndian>(self.header.section_header_num);
        cursor.write_u16::<LittleEndian>(self.header.section_header_str_index);

        // Write ELF Program Header
        cursor.write_u32::<LittleEndian>(self.program_header.program_header_type);
        cursor.write_u32::<LittleEndian>(self.program_header.program_header_flags);
        cursor.write_u64::<LittleEndian>(self.program_header.program_header_offset);
        cursor.write_u64::<LittleEndian>(self.program_header.program_header_vaddr);
        cursor.write_u64::<LittleEndian>(self.program_header.program_header_paddr);
        cursor.write_u64::<LittleEndian>(self.program_header.program_header_filesz);
        cursor.write_u64::<LittleEndian>(self.program_header.program_header_memsz);
        cursor.write_u64::<LittleEndian>(self.program_header.program_header_align);

        // Write ELF Section
        Write::write_all(&mut cursor, &self.sections.body).unwrap();

        elf_bytes
    }

}
