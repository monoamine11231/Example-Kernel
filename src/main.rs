#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(strict_provenance)]
#![feature(ptr_from_ref)]
#![feature(abi_x86_interrupt)]
#![allow(unused, unconditional_panic)]

#[macro_use]
extern crate lazy_static;

mod bord;
mod drivers;
mod handlers;
pub mod mem;
mod pic;
mod tooling;
use core::arch::asm;
use core::fmt::Write;
use core::str::Bytes;

use bord::*;
use drivers::ide::IDE;
pub mod fat32;
use heapless::String;
use mem::memory::{self, *};
use tooling::qemu_io::{qemu_print_hex, qemu_println};
use tooling::vga::write_str_at;

use crate::handlers::pic_intr_handler;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() -> ! {
    load_idt(&IDTX);
    memory::init();
    //pic::init();

    let buf: [u8; 10] = [0x10u8; 10];


    let mut ide_processor: IDE = Default::default();
    ide_processor.init();
    let mut fs_processor = fat32::FAT32::new(&mut ide_processor).unwrap();
    qemu_print_hex(fs_processor.traverse("KEK/ABA/LOL3.TXT").unwrap().is_some() as u32);
    

    //qemu_print_hex(a);

    write_str_at("Hello World!", 0, 0, 0xb);

    // unsafe {
    //     asm!(
    //         "div {0:e}",
    //         in(reg) 0,
    //     )
    // }

    loop {}
}

lazy_static! {
    static ref IDTX: IDT = IDT {
        page_fault: IDTEntry::new(handlers::page_fault, Ring::Zero),
        divide_error: IDTEntry::new(handlers::zero_div, Ring::Zero),
        debug: IDTEntry::new(handlers::debug, Ring::Zero),
        non_maskable_interrupt: IDTEntry::new(handlers::non_maskable_interrupt, Ring::Zero),
        breakpoint: IDTEntry::new(handlers::breakpoint, Ring::Zero),
        overflow: IDTEntry::new(handlers::overflow, Ring::Zero),
        bound_range_exceeded: IDTEntry::new(handlers::bound_range_exceeded, Ring::Zero),
        invalid_opcode: IDTEntry::new(handlers::invalid_opcode, Ring::Zero),
        device_not_available: IDTEntry::new(handlers::device_not_available, Ring::Zero),
        double_fault: IDTEntry::new(handlers::double_fault, Ring::Zero),
        invalid_tss: IDTEntry::new(handlers::invalid_tss, Ring::Zero),
        segment_not_present: IDTEntry::new(handlers::segment_not_present, Ring::Zero),
        stack_segment_fault: IDTEntry::new(handlers::stack_segment_fault, Ring::Zero),
        general_protection_fault: IDTEntry::new(handlers::general_protection_fault, Ring::Zero),
        x87_floating_point: IDTEntry::new(handlers::x87_floating_point, Ring::Zero),
        alignment_check: IDTEntry::new(handlers::alignment_check, Ring::Zero),
        machine_check: IDTEntry::new(handlers::machine_check, Ring::Zero),
        simd_floating_point: IDTEntry::new(handlers::simd_floating_point, Ring::Zero),
        virtualization: IDTEntry::new(handlers::virtualization, Ring::Zero),
        security_exception: IDTEntry::new(handlers::security_exception, Ring::Zero),
        interrupts: [IDTEntry::new(pic_intr_handler, Ring::Zero); 16],
        ..Default::default()
    };
}

fn panicking_function() -> ! {
    //write_str_at("Panicking function call", 0, 0, 0xb);
    //tooling::panic_handler::stack_trace();
    panic!("This is a test panic.");

    loop {}
}
