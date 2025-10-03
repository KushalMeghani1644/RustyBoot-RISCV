#![no_std]
#![no_main]
#![feature(asm)]

mod block;
mod bootloader;
mod elf;
mod fs;
mod memory;
mod uart;

use memory::frame_allocator;
use uart::*;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Init UART first for debugging
    uart::init();
    uart::println("=== RustyBoot-RISCV ===");

    // Init memory/frame allocator
    uart::println("Init memory");
    frame_allocator::reserve_region(0x8000_0000, 0x8100_0000); // Reserve bootloader & low memory
    uart::println("Memory reserved for bootloader");

    // Allocate some text frames
    uart::println("Allocating test frames");
    for _ in 0..5 {
        if let Some(frame) = frame_allocator::allocate_frame() {
            uart::prinln("Allocated frame at 0x{:x}", frame);
        } else {
            uart::println("Out of memory");
        }
    }

    // Debug: print all used frames
    frame_allocator::debug_print();

    // Load the kernel
    uart::println("Loading kernel...");
    bootloader::load_kernel();

    // Should never reach here.
    loop {}
}
