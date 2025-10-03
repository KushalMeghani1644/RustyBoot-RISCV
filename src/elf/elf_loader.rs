#![no_std]

use core::ptr;

#[repr(C)]
pub struct Elf64Header {
    e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

pub const PT_LOAD: u32 = 1;

/// Load an ELF binary from memory and return entrypoint
pub unsafe fn load_elf(elf_start: *const u8) -> fn() -> ! {
    let header = &*(elf_start as *const Elf64Header);

    // Basic validation
    if &header.e_ident[0..4] != b"\x7FELF" {
        loop {} // Invalid Elf, hang
    }

    let entry = header.e_entry as usize;

    // Load program headers
    let phdr_start = (elf_start as usize + header.e_phoff as usize) as *const ProgramHeader;
    for i in 0..header.e_phnum {
        let ph = &*phdr_start.add(i as usize);
        if ph.p_type != PT_LOAD {
            continue;
        }

        let dest = ph.p_paddr as *mut u8;
        let src = (elf_start as usize + ph.p_offset as usize) as *const u8;

        // Copy bytes from ELF to memory
        for j in 0..ph.p_filesz as usize {
            ptr::write_volatile(dest.add(j), ptr::read_volatile(src.add(j)));
        }

        /// Zero initialize remaining memory
        for j in ph.p_filesz as usize..ph.p_memsz {
            ptr::write_volatile(dest.add(j), 0);
        }
    }
    core::mem::transmute::<usize, fn() -> !>(entry)
}
