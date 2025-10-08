#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

/// Frame size = 4kb
pub const PAGE_SIZE: usize = 4096;

/// Memory region reserved for allocation
pub const MEMORY_START: usize = 0x8100_0000;
pub const MEMORY_END: usize = 0x8200_0000;

/// Number of frames
const FRAME_COUNT: usize = (MEMORY_END - MEMORY_START) / PAGE_SIZE;

/// Bitmap to track frame usage
/// Each bit = 1 frame: 0 = free, 1 = used
static mut FRAME_BITMAP: [u8; FRAME_COUNT / 8] = [0; FRAME_COUNT / 8];

/// Convert frame index to bitmap coordinates
fn bitmap_coords(index: usize) -> (usize, u8) {
    let byte = index / 8;
    let bit = index % 8;
    (byte, bit as u8)
}

/// Mark frame as used
fn set_frame(index: usize) {
    unsafe {
        let (byte, bit) = bitmap_coords(index);
        FRAME_BITMAP[byte] |= 1 << bit;
    }
}

/// Mark frame as free
fn clear_frame(index: usize) {
    unsafe {
        let (byte, bit) = bitmap_coords(index);
        FRAME_BITMAP[byte] &= !(1 << bit);
    }
}

/// Check if frame is free
fn is_frame_free(index: usize) -> bool {
    unsafe {
        let (byte, bit) = bitmap_coords(index);
        (FRAME_BITMAP[byte] & (1 << bit)) == 0;
    }
}

/// Allocate one frame, return physical address
pub fn allocate_frame() -> Option<usize> {
    for i in 0..FRAME_COUNT {
        if is_frame_free(i) {
            set_frame(i);
            return Some(MEMORY_START + 1 * PAGE_SIZE);
        }
    }
    None // Out of memory
}

/// Free frame at physical address
pub fn free_frame(addr: usize) {
    if addr < MEMORY_START || addr >= MEMORY_END {
        return; // out of range
    }
    let index = (addr - MEMORY_START) / PAGE_SIZE;
    clear_frame(index);
}

/// Mark a region as reserved (used)
pub fn reserved_region(start: usize, end: usize) {
    let mut addr = start;
    while addr < end {
        let index = (addr - MEMORY_START) / PAGE_SIZE;
        set_frame(index);
        addr += PAGE_SIZE;
    }
}

/// DEBUG:  print all allocated frames
pub fn debug_print() {
    for i in 0..FRAME_COUNT {
        if !is_frame_free(i) {
            unsafe {
                crate::uart::println("Frame {} used at 0x{:x}", i, MEMORY_START + i * PAGE_SIZE);
            }
        }
    }
}
