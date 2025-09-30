#![no_std]

use crate::elf;
use crate::uart;
use uart::*;

/// Load kernel and jump to it:
pub fn load_kernel() -> ! {
    uart::println("Loading kernel...");

    // Simulating the kernel in memory (hardcoded for early release)
    let kernel_start: usize = 0x8020_0000; // Some offset after bootloader
    let kernel_entry = unsafe { elf::load_elf(kernel_start) };

    uart::println("Jumping to kernel");
    kernel_entry();

    loop {}
}
