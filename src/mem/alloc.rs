use crate::mem::memory;
use crate::tooling::qemu_io::{qemu_fmt_println, qemu_print_hex, qemu_println};
use core::mem;
use core::mem::size_of;
use core::ptr;

use core::ptr::{addr_of_mut, *};
// do not allocate over regions referenced by e820

#[repr(C)]
pub struct Allocator {
    pub head: *mut u8,
}

// how to singleton ?????? (with custom bootloader)
impl Allocator {
    pub unsafe fn new() -> &'static mut Self {
        let mut adata = ((1 << 30) + 0xFFFFF + 1) as *mut Allocator;
        (*adata).head = (adata as u64 + (size_of::<Self>() as u64)) as *mut u8;
        &mut *adata
    }
    pub unsafe fn get() -> &'static mut Self {
        let mut adata = ((1 << 30) + 0xFFFFF + 1) as *mut Allocator;
        &mut *adata
    }
}

pub unsafe fn init_alloc() {
    Allocator::new(); // figure out singletons.
    alloc_test();
}

fn alloc_test() {
    unsafe {
        let ptra = kalloc(512);
        //qemu_fmt_println("{}", format_args!("{:#x}", ptra as u64));
        let ptrb = kalloc(512);
        assert!((ptrb as u64) - (ptra as u64) == 0x200);
        //qemu_fmt_println("{}", format_args!("{:#x}", ptrb as u64));
        //qemu_fmt_println("{}", format_args!("{:#x}", (ptrb as u64) - (ptra as u64)));
    }
}

pub fn kalloc(size: usize) -> *mut u8 {
    unsafe { kalloc_linear(size) }
}

pub fn kalloc_linear(size: usize) -> *mut u8 {
    unsafe {
        let mut allocator = Allocator::get();
        allocator.head = (allocator.head as u64 + size as u64) as *mut u8;
        allocator.head
    }
}
