# RustyBoot-RISC-V

Note: This project is just started and is WIP! Please use a VM for testing.

---

RustyBoot-RISC-V is a **minimalist, modular bootloader for RISC-V platforms**, inspired by the original RustyBoot UEFI bootloader for x86. It aims to provide a flexible foundation for loading RISC-V kernels directly from block devices with support for MBR/GPT partition tables and EXT2/3/4 filesystems.

---

## Features (Implemented / Planned)

**Implemented**
- Bare-Metal RISCV `_start()` entry point
- UART driver for early debug output
- Minimal ELF kernel loader
- Bootloader skeleton structure
  
**Planned**
- Memory manager with frame allocator
- Block device abstraction (virtio for QEMU, SD card support)
- MBR/GPT partitioning parsing
- EXT2/3/4 filesystem support
- kernel loading from disk

---

## Getting Started

### Requirements

- Rust nightly with `cargo-xbuild`
- `rustup target add riscv64imac-unknown-none-elf`
- QEMU for RISC-V emulation (optional for testing)

### Build

```bash
# Build the bootloader
cargo build --target riscv64imac-unknown-none-elf
```

### Run in QEMU

```bash
qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios none \
    -kernel target/riscv64imac-unknown-none-elf/debug/rustyboot-riscv \
    -serial mon:stdio
```

-You should see UART output in the terminal indicating boot progress.

### Project Structure

```
RustyBoot-RISCV/

├── src/
│   ├── main.rs
│   ├── panic.rs
│   ├── uart/
│   ├── memory/
│   ├── block/
│   ├── fs/
│   └── elf/
├── linker.ld
└── boot/               
```

### Contributing

Contributions are welcome! Please follow the modular structure and maintain `no_std` compatibility for all bootloader components. Open issues or pull requests for:

- Device driver improvements

- Filesystem enhancements

- Kernel loader optimizations

- New RISC-V platform support

### License

This project is licensed under the GPLv3 license. See [LICENSE](RustyBoot-RISCV/LICENSE) for details.

### _Built With ❤️ in Rust_
