#![no_std]
#![no_main]
#![feature(asm)]

mod block;
mod bootloader;
mod elf;
mod fs;
mod memory;
mod uart;
use uart::*;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Init UART first for debugging
    uart::init();

    uart::println("=== RustyBoot-RISCV ===");

    bootloader::load_kernel();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    uart::println("PANIC!");
    loop {}
}
