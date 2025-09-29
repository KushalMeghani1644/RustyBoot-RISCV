#![no_std]
#![no_main]
#![feature(asm)]

mod uart;
mod block;
mod memory;
mod fs;
mod elf;

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Init UART first for debugging
    uart::init();

    uart::println("=== RustyBoot-RISCV ===");

    // Init memory manager
    memory::init();

    // Init block device (SD card/ virtio)
    block::init();

    // Load boot info (MBR/GPT)
    let boot_info = fs::load_boot_info().unwrap_or_else(|| {
        uart::println("Failed to load boot info!");
        loop {}
    });

    uart::println("Boot info loaded");

    // Loaded kernel elf
    match elf::load_kernel(&boot_info.kernel_path) {
        Ok(entry_point) => {
            uart::println("Kernel Loaded, jumping now");
            jump_to_kernel(entry_point);
        }
        Err(_) => {
            uart::println("Failed to load kernel");
            loop {}
        }
    }
    loop {}
}

fn jump_to_kernel(entry: usize) -> ! {
    unsafe {
        // Jump to kernel entry point
        let entry_fn: extern "C" fn() -> ! core::mem::transmute(entry);
        entry_fn();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    uart::println("PANIC!");
    if let Some(location) = info.location() {
        uart::println(&format!("{}:{}", location.file(), location.line()));
    }
  loop {}
}
