use crate::graph::planar::*;
use crate::graph::planar_writer::*;
use crate::graph::utils::*;
use crate::utils::qemu_io::{
    self, qemu_print_nln, qemu_print_num, qemu_print_str, qemu_print_u8, qemu_println,
};
use core::arch::asm;

pub struct Window {
    width: usize,
    height: usize,
    id: u8,
}

impl Window {
    //static mut planar_writer : VGA_planar_writer = VGA_planar_writer::new();
}
