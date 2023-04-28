use core::panic::PanicInfo;

use crate::tooling::{
    self,
    qemu_io::{qemu_print_hex, qemu_println},
    serial::{inb, outb},
    vga::write_str_at,
};

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

pub extern "x86-interrupt" fn pic_intr_handler(isf: InterruptStackFrame) {
    // wrong args i think

    let scancode = inb(0x60);
    qemu_print_hex(scancode as u32);

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

// blatantly stolen struct
#[repr(C)]
pub struct InterruptStackFrame {
    pub instruction_ptr: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}
