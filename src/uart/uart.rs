#![no_std]

const UART0_BASE: usize = 0x1000_0000; // QEMU virt memory-mapped UART0

pub fn init() {
    // UART initialization for QEMU virt is minimal: nothing required
    // Can add custom baud divisor setup if needed later
}

fn uart_tx(c: u8) {
    unsafe {
        // Wait for UART to be ready
        let uart_tdr = (UART0_BASE + 0x00) as *mut u8; // Transmit Data Register
        core::ptr::write_volatile(uart_tdr, c);
    }
}

pub fn putchar(c: char) {
    uart_tx(c as u8);
}

pub fn print(s: &str) {
    for c in s.chars() {
        if c == '\n' {
            putchar('\r');
        }
        putchar(c);
    }
}

pub fn println(s: &str) {
    print(s);
    print("\n");
}
