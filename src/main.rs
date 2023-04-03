#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(strict_provenance)]
#![feature(abi_x86_interrupt)]
#![allow(unused, unconditional_panic)]

#[macro_use]
extern crate lazy_static;

mod bord;
mod handlers;
mod tooling;

use core::{arch::asm, panic};

use bord::*;
use tooling::vga::write_str_at;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() -> ! {
    load_idt(&IDTX);
    write_str_at("Hello World!", 0, 0, 0xb);

    loop {}
}

lazy_static! {
    static ref IDTX: IDT = IDT {
        page_fault: IDTEntry::new(handlers::page_fault, Ring::Zero),
        divide_by_zero: IDTEntry::new(handlers::zero_div, Ring::Zero),
        ..Default::default()
        // debug: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // non_maskable_interrupt: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // breakpoint: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // overflow: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // bound_range_exceeded: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // invalid_opcode: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // device_not_available: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // double_fault: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // invalid_tss: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // segment_not_present: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // stack_segment_fault: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // general_protection_fault: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // x87_floating_point: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // alignment_check: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // machine_check: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // simd_floating_point: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // virtualization: IDTEntry::new(handlers::page_fault, Ring::Zero),
        // security_exception: IDTEntry::new(handlers::page_fault, Ring::Zero),
    };
}

fn panicking_function() -> ! {
    //write_str_at("Panicking function call", 0, 0, 0xb);
    //tooling::panic_handler::stack_trace();
    panic!("This is a test panic.");

    loop {}
}
