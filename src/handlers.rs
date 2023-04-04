use core::panic::PanicInfo;

use crate::tooling::{self, vga::write_str_at};

pub extern "x86-interrupt" fn page_fault() {
    write_str_at("err: page fault", 4, 0, 0xde)
}

pub extern "x86-interrupt" fn zero_div() {
    write_str_at("err: div zero", 5, 0, 0xde)
}

#[macro_export]
macro_rules! interrupt_asd {
    ($x:tt,$a:tt) => {
        pub extern "x86-interrupt" fn $x() {
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
interrupt_asd!(double_fault, 8);
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
