use core::arch::asm;

use crate::acpi::*;

lazy_static! {
    pub static ref MADTX: MADT = RSDTX.find_table::<MADT>().unwrap();
}

impl Signature for MADT {
    fn get_signature() -> [u8; 4] {
        *b"APIC"
    }
}

// https://wiki.osdev.org/MADT
#[repr(C)]
pub struct MADT {
    pub signature: u32,
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oemid: [u8; 6],
    pub oem_table_id: u64,
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

pub fn init() {
    disable_pic();
}
// https://wiki.osdev.org/PIC#Disabling
fn disable_pic() {
    unsafe {
        asm!(
            // // init the pics
            "mov al, 0x11",
            "out 0x20, al",
            "out 0xa0, al",
            // move the offsets to 32 and 40
            "mov al, 0x20",
            "out 0x21, al",
            "mov al, 0x28",
            "out 0xa1, al",
            // chain stuff
            "mov al, 0x04",
            "out 0x21, al",
            "mov al, 0x02",
            "out 0xa1, al",
            // mode x801239384
            "mov al, 0x01",
            "out 0x21, al",
            "out 0xa1, al",
            // mask
            "mov al, 0xff",
            "out 0xa1, al",
            "out 0x21, al",
            options(nostack, preserves_flags, nomem)
        );
    }
}
