use core::{arch::asm, mem::size_of};

use crate::tooling::qemu_io::{qemu_print_hex, qemu_println, SerialWriter};
use core::fmt::Write;

lazy_static! {
    pub static ref RSDPX: &'static RSDP = find_rsdp().unwrap();
    pub static ref RSDTX: &'static RSDT = RSDT::from_rsdp(&RSDPX).unwrap();
}

macro_rules! sum_struct {
    ($struct:expr) => {{
        let mut sum = 0_u8;
        let ptr = $struct as *const _ as *const u8;
        let size = core::mem::size_of_val($struct);
        let bytes = unsafe { core::slice::from_raw_parts(ptr, size) };
        for byte in bytes {
            sum = sum.wrapping_add(*byte);
        }
        sum
    }};
}

// https://wiki.osdev.org/RSDP
#[repr(C, packed)]
pub struct RSDP {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oemid: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    // qemu doesn't use v2 it seems

    // pub length: u32,
    // pub xsdt_address: u64,
    // pub extended_checksum: u8,
    // pub reserved: [u8; 3],
}

impl RSDP {
    const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";

    pub fn validate(&self) -> bool {
        self.signature.eq(Self::RSDP_SIGNATURE) && sum_struct!(self) == 0
    }
}

#[repr(C, packed)]
#[derive(Debug)]

pub struct SDTHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

#[repr(C, packed)]
pub struct RSDT {
    pub h: SDTHeader,
}

pub trait Signature {
    fn get_signature() -> [u8; 4];
}

impl RSDT {
    fn from_rsdp(rsdp: &RSDP) -> Option<&RSDT> {
        qemu_print_hex(rsdp.rsdt_address);
        let res = rsdp.rsdt_address as *const RSDT;
        let res = unsafe { &*res };

        // panic since it is a physical addr

        if !res.is_valid() {
            return None;
        }

        Some(res)
    }

    pub fn find_table<A>(&self) -> Option<A>
    where
        A: Signature,
    {
        let entries = (RSDTX.h.length - size_of::<RSDT>() as u32) / 4;

        for x in 0..entries {}
        None
    }

    pub fn is_valid(&self) -> bool {
        self.h.signature.eq(b"RSDT") && sum_struct!(self) == 0
    }
}

fn find_rsdp() -> Result<&'static RSDP, &'static str> {
    let bios_start = 0x000E_0000;
    let bios_end = 0x0010_0000;

    for addr in (bios_start..bios_end).step_by(16) {
        let ptr = addr as *const RSDP;
        let rsdp = unsafe { &*ptr };

        if !rsdp.validate() {
            continue;
        }

        if rsdp.revision != 0 {
            return Err("expected RSDP version to be 1"); // expect the unexpected
        }

        return Ok(rsdp);
    }

    Err("could not find rsdp")
}

pub fn qemu_shutdown() -> ! {
    // reference https://wiki.osdev.org/Shutdown
    unsafe {
        asm!(
            "mov dx, 0x604",
            "mov ax, 0x2000",
            "out dx, ax",
            options(nostack, preserves_flags, noreturn, nomem)
        );
    };
    loop {
        qemu_println("why haven't we shut down yet?");
    }
}
