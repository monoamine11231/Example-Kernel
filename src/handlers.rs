use core::panic::PanicInfo;

use crate::tooling::{self, vga::write_str_at};

pub extern "x86-interrupt" fn page_fault() {
    write_str_at("err: page fault", 4, 3, 0xde)
}

pub extern "x86-interrupt" fn zero_div() {
    write_str_at("err: div zero", 5, 3, 0xde)
}
