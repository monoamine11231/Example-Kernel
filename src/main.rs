#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod tooling;
use core::arch::asm;
use core::fmt::Write;

use tooling::vga::write_str_at;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut console_writer = tooling::vga::VGAWriter::new();
    write!(console_writer, "Hello World!").unwrap();
    inline_assmebly_test(&mut console_writer);
    //panicking_function(&mut console_writer);
    loop {}
}

fn write_to_linear_buffer(test_arg: &mut u64, writer: &mut tooling::vga::VGAWriter) {
    let mut t: u64;
    let mut q: u64;

    t = 5;
    q = 6;
    let mut c = t + q;
    (*test_arg) = c;
    inline_assmebly_test(writer);
}

fn inline_assmebly_test(writer: &mut tooling::vga::VGAWriter) {
    let mut saved_rbp: u64;
    let mut saved_rsp: u64;

    unsafe {
        asm!("
            mov {0}, rbp
            mov {1}, rsp
        ", out(reg) saved_rbp, out(reg) saved_rsp)
    }

    write!(writer, "{}", format_args!("RBP = {:#x}", saved_rbp)).unwrap();
    writer.newline();
    write!(writer, "{}", format_args!("RSP = {:#x}", saved_rsp)).unwrap();
    writer.newline();
}

fn panicking_function(writer: &mut tooling::vga::VGAWriter) {
    //write_str_at("Panicking function call", 0, 0, 0xb);
    tooling::panic_handler::stack_trace(writer);
    //panic!("This is a test panic.");
}
