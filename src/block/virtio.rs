#![no_std]

use core::ptr::{read_volatile, write_volatile};

const VIRTIO_MMIO_BASE: usize = 0x1000_2000;

// MMIO register offset
const MAGIC_VALUE: usize = 0x000;
const VERSION: usize = 0x004;
const DEVICE_ID: usize = 0x008;
const VENDOR_ID: usize = 0x00c;

const DEVICE_FEATURES: usize = 0x010;
const DRIVER_FEATURS: usize = 0x020;

const QUEUE_SEL: usize = 0x030;
const QUEUE_NUM_MAX: usize = 0x034;

const QUEUE_NUM: usize = 0x038;
const QUEUE_READY: usize = 0x044;
const QUEUE_DESC_LOW: usize = 0x080;
const QUEUE_DESC_HIGH: usize = 0x084;
const QUEUE_AVAIL_LOW: usize = 0x090;
const QUEUE_AVAIL_HIGH: usize = 0x094;
const QUEUE_USED_LOW: usize = 0x0a0;
const QUEUE_USED_HIGH: usize = 0x0a4;

const STATUS: usize = 0x070;
const QUEUE_NOTIFY: usize = 0x050;
const INTERRUPT_STATUS: usize = 0x060;
const INTERRUPT_ACK: usize = 0x064;

const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_DRIVER_OK: u32 = 4;
const STATUS_FEATURES_OK: u32 = 8;
const STATUS_FAILED: u32 = 0x80;

const VIRTIO_BLK_DEVICE: u32 = 2;
const VIRTIO_MAGIC: u32 = 0x7472_6976;

const DRIVER_FEATURES_SEL: u32 = 0;

const VRING_DESC_F_NEXT: u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;

#[repr(C, align(4096))]
struct Aligned([u8; 4096]);

static mut DESC_AREA: Aligned = Aligned([0u8; 4096]);
static mut AVAIL_AREA: Aligned = Aligned([0u8; 4096]);
static mut USED_AREA: Aligned = Aligned([0u8; 4096]);

#[repr(C, align(4096))]
struct ReqBuf([u8; 4096]);

static mut REQUEST_BUFFER: ReqBuf = ReqBuf([0u8; 4096]);
static mut STATUS_BYTE: [u8; 1] = [0u8; 1];

#[repr(C)]
struct VirtioBlkRed {
    // type: 0=read, 1=write, 2=flush
    req_type: u32,
    reserved: u32,
    sector: u64,
}

const VIRTIO_BLK_T_IN: u32 = 0; // read
const VIRTIO_BLK_T_OUT: u32 = 1; // write

/// Low-level MMIO helpers
fn mmio_read32(offset: usize) -> u32 {
    unsafe { read_volatile((VIRTIO_MMIO_BASE + offset) as *const u32) }
}

fn mmio_write32(offset: usize, v: u32) {
    unsafe { write_volatile((VIRTIO_MMIO_BASE + offset) as *const u32, v) }
}

fn mmio_read64(offset: usize) -> u64 {
    let low = mmio_read32(offset) as u64;
    let high = mmio_read32(offset + 4) as u64;
    (high << 32) | low
}

fn mmio_write64(offset: usize, v: u64) {
    mmio_write32(offset, v as u32);
    mmio_write32(offset + 4, (v >> 32) as u32);
}

/// Basic probe: check magic, version, device id
pub fn probe() -> bool {
    let magic = mmio_read32(MAGIC_VALUE);
    let version = mmio_read32(VERSION);
    let device = mmio_read32(DEVICE_ID);

    if magic == VIRTIO_MAGIC && (version == 1 || version == 2) && device == VIRTIO_BLK_DEVICE {
        crate::uart::println("virtio: device detected");
        true
    } else {
        crate::uart::println("virtio: device not found");
        false
    }
}

pub fn init() -> bool {
    mmio_write32(STATUS, STATUS_ACKNOWLEDGE);
    mmio_write32(STATUS, STATUS_ACKNOWLEDGE | STATUS_DRIVER);

    // read device features (we will accept none)
    let dev_feats = mmio_read32(DEVICE_FEATURES);
    let _ = dev_feats;

    // write driver features (0)
    mmio_write32(DRIVER_FEATURES, 0);

    // indicate features ok
    mmio_write32(
        STATUS,
        STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK,
    );

    let status_after_features = mmio_read32(STATUS);
    if (status_after_features & STATUS_FAILED) != 0 {
        crate::uart::println("virtio: device rejected features");
        return false;
    }

    // Select queue 0
    mmio_write32(QUEUE_SEL, 0);
    let qmax = mmio_read32(QUEUE_NUM_MAX);
    if qmax == 0 {
        crate::uart::println("virtio: queue 0 not available");
        return false;
    }
    let qsize = if qmax >= 8 { 8 } else { qmax };
    mmio_write32(QUEUE_NUM, qsize);

    unsafe {
        let desc_paddr = &DESC_AREA as *const _ as u64;
        let avail_paddr = &AVAIL_AREA as *const _ as u64;
        let used_paddr = &USED_AREA as *const _ as u64;

        mmio_write32(QUEUE_DESC_LOW, desc_paddr as u32);
        mmio_write32(QUEUE_DESC_HIGH, (desc_paddr >> 32) as u32);

        mmio_write32(QUEUE_AVAIL_LOW, avail_paddr as u32);
        mmio_write32(QUEUE_AVAIL_HIGH, (avail_paddr >> 32) as u32);

        mmio_write32(QUEUE_USED_LOW, used_paddr as u32);
        mmio_write32(QUEUE_USED_HIGH, (used_paddr >> 32) as u32);
    }

    // mark queue ready
    mmio_write32(QUEUE_READY, 1);

    // driver ready
    let final_status = STATUS_ACKNOWLEDGE | STATUS_DRIVER | STATUS_FEATURES_OK | STATUS_DRIVER_OK;
    mmio_write32(STATUS, final_status);

    crate::uart::println("virtio: init done");
    true
}

/// Notify queue (write index)
fn notify_queue(_queue: u32) {
    mmio_write32(QUEUE_NOTIFY, _queue);
}

/// A very small helper for busy-waiting
fn poll_used_ring() -> bool {
    // check interrupt status or used ring contents
    // For simplicity, we'll poll INTERRUPT_STATUS / USED area
    // (Used ring index updating is device-specific; we'll read used->idx)
    let used_idx = unsafe {
        // used structure (per vring):
        // u32 flags; u32 idx; used_elem[]
        // We stored it in USED_AREA
        let used_ptr = (&USED_AREA.0) as *const u8 as *const u32;
        // idx is at offset 4 bytes
        read_volatile(used_ptr.add(1))
    };
    used_idx > 0
}

/// Public API: read a single 512-byte block (LBA) into buf
pub fn read_block(lba: u64, buf: &mut [u8; 512]) -> Result<(), &'static str> {
    unsafe {
        let hdr_ptr = (&mut REQUEST_BUFFER.0[0]) as *mut u8 as *mut VirtioBlkReq;
        (*hdr_ptr).req_type = VIRTIO_BLK_T_IN;
        (*hdr_ptr).reserved = 0;
        (*hdr_ptr).sector = lba;

        let data_ptr = (&mut REQUEST_BUFFER.0[core::mem::size_of::<VirtioBlkReq>()]) as *mut u8;

        for i in 0..512 {
            write_volatile(data_ptr.add(i), 0u8);
        }

        /// Status byte will be separate above STATUS_BYTE
        STATUS_BYTE[0] = 0;

        /// Build descriptors in the DESC_AREA for a 3-descriptor chain:
        /// desc0: header (device-readable)
        /// desc1: data buffer (device-writes for read)
        /// desc2: status byte (device-writes)
        ///
        /// vnode vring descriptor struct layout:
        /// u64 addr; u32 len; u16 flags; u16 next;
        #[repr(C)]
        struct Vdesc {
            addr: u64,
            len: u32,
            flags: u16,
            next: u16,
        }

        let descs = (&mut DESC_AREA.0) as *mut u8 as *mut Vdesc;
        // zero descriptors
        core::ptr::write_bytes(descs as *mut u8, 0, core::mem::size_of::<Vdesc>() * 8);

        // addresses:
        let hdr_addr = (&REQUEST_BUFFER.0[0]) as *const u8 as u64;
        let data_addr = data_ptr as *const u8 as u64;
        let status_addr = (&STATUS_BYTE[0]) as *const u8 as u64;

        (*descs.add(0)).addr = hdr_addr;
        (*descs.add(0)).len = core::mem::size_of::<VirtioBlkReq>() as u32;
        (*descs.add(0)).flags = VRING_DESC_F_NEXT;
        (*descs.add(0)).next = 1;

        (*descs.add(1)).addr = data_addr;
        (*descs.add(1)).len = 512;
        (*descs.add(1)).flags = VRING_DESC_F_NEXT | VRING_DESC_F_WRITE;
        (*descs.add(1)).next = 2;

        (*descs.add(2)).addr = status_addr;
        (*descs.add(2)).len = 1;
        (*descs.add(2)).flags = VRING_DESC_F_WRITE;
        (*descs.add(2)).next = 0;

        let avail_ptr = (&mut AVAIL_AREA.0) as *mut u8 as *mut u16;
        // flags = 0
        write_volatile(avail_ptr.add(0), 0);
        // idx = 1 (we put one descriptor)
        write_volatile(avail_ptr.add(1), 1u16);
        // ring[0] = descriptor head (0)
        notify_queue(0);
        let mut tries = 0u32;
        loop {
            let used_idx_ptr = (&USED_AREA.0) as *mut u8 as *mut u32;
            let idx = read_volatile(used_idx_ptr.add(1));
            if idx > 0 {
                if STATUS_BYTE[0] != 0 {
                    return Err("virtio: read status error");
                }
                for i in 0..512 {
                    let b = read_volatile(data_ptr.add(i));
                    buf[i] = b;
                }
                return Ok(());
            }
            tries = tries.wrapping_add(1);
        }
    }
}
