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

    let mut hej = 1;
    unsafe {
        asm!(
            "div {0:e}",
            in(reg) 0,
            inout("eax") hej
        )
    }

    loop {}
}

lazy_static! {
    static ref IDTX: IDT = IDT {
        page_fault: IDTEntry::new(handlers::page_fault, Ring::Zero),
        divide_by_zero: IDTEntry::new(handlers::zero_div, Ring::Zero),
        ..IDT::default()
    };
}

fn panicking_function() -> ! {
    //write_str_at("Panicking function call", 0, 0, 0xb);
    //tooling::panic_handler::stack_trace();
    panic!("This is a test panic.");

    loop {}
}
