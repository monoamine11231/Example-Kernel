use core::panic::PanicInfo;
use crate::{time, qemu_print};

use crate::tooling::{
    self,
    qemu_io::{qemu_print_hex, qemu_println},
    serial::{inb, outb},
    vga::write_str_at,
};

static mut TIME_ELAPSED: u64 = 0;

pub extern "x86-interrupt" fn page_fault(isf: InterruptStackFrame) {
    write_str_at("err: page fault", 4, 0, 0xde)
}

pub extern "x86-interrupt" fn zero_div(isf: InterruptStackFrame) {
    qemu_println("inst ptr: ");
    qemu_print_hex(isf.stack_pointer as u32);
    qemu_print_hex((isf.stack_pointer >> 32) as u32);

    write_str_at("err: div zero", 5, 0, 0xde)
}

pub extern "x86-interrupt" fn double_fault(isf: InterruptStackFrame) {
    write_str_at("err: reee", 5, 0, 0xde)
}

pub extern "x86-interrupt" fn keyboard_handler(isf: InterruptStackFrame) {
    // wrong args i think

    let scancode = inb(0x60);
    qemu_print_hex(scancode as u32);

    outb(0xA0, 0x20);
    outb(0x20, 0x20);
}

pub extern "x86-interrupt" fn mystery_pic_intr_handler(isf: InterruptStackFrame) {
    // wrong args i think

    qemu_println("hej");

    outb(0xA0, 0x20);
    outb(0x20, 0x20);
}

#[macro_export]
macro_rules! mystery_handler {
    ($x:tt) => {
        pub extern "x86-interrupt" fn $x(isf: InterruptStackFrame) {
            qemu_println(concat!("mystery: ", stringify!($x)));
            outb(0xA0, 0x20);
            outb(0x20, 0x20);
        }
    };
}

pub extern "x86-interrupt" fn handler1_wtf(isf: InterruptStackFrame) {
    // wrong args i think
    unsafe {
        time::MILLIS += 1;
        if time::MILLIS == 1000 {
            time::MILLIS = 0;
            TIME_ELAPSED += 1;
            qemu_print!("a second passed! ({})\n", TIME_ELAPSED);
        }
    }
    outb(0xA0, 0x20);
    outb(0x20, 0x20);
    
}

#[macro_export]
macro_rules! interrupt_asd {
    ($x:tt,$a:tt) => {
        pub extern "x86-interrupt" fn $x(isf: InterruptStackFrame) {
            write_str_at(concat!("err: ", stringify!($x)), $a, 0, 0xde)
        }
    };
}

interrupt_asd!(debug, 1);
interrupt_asd!(non_maskable_interrupt, 2);
interrupt_asd!(breakpoint, 3);
interrupt_asd!(overflow, 4);
interrupt_asd!(bound_range_exceeded, 5);
interrupt_asd!(invalid_opcode, 6);
interrupt_asd!(device_not_available, 7);
interrupt_asd!(invalid_tss, 9);
interrupt_asd!(segment_not_present, 10);
interrupt_asd!(stack_segment_fault, 11);
interrupt_asd!(general_protection_fault, 12);
interrupt_asd!(x87_floating_point, 13);
interrupt_asd!(alignment_check, 14);
interrupt_asd!(machine_check, 2);
interrupt_asd!(simd_floating_point, 3);
interrupt_asd!(virtualization, 4);
interrupt_asd!(security_exception, 5);

mystery_handler!(mh1);
mystery_handler!(mh3);
mystery_handler!(mh4);
mystery_handler!(mh5);
mystery_handler!(mh6);
mystery_handler!(mh7);
mystery_handler!(mh8);
mystery_handler!(mh9);
mystery_handler!(mh10);
mystery_handler!(mh11);
mystery_handler!(mh12);
mystery_handler!(mh13);
mystery_handler!(mh14);
mystery_handler!(mh15);
mystery_handler!(mh16);

// blatantly stolen struct
#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_ptr: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}
