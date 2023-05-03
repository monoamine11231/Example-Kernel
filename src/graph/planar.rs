use crate::graph::utils::*;
use crate::tooling::qemu_io::*;
use crate::tooling::serial::{outb, outw};
use core::arch::asm;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Planar {
    current_planar: usize,
}

impl Planar {
    const CODE_MAPPING: [u16; 4] = [0x102, 0x202, 0x402, 0x802];

    pub fn new() -> Self {
        Planar::indicate_planar_switch();
        Planar::plane_0();
        Planar { current_planar: 0 }
    }

    pub fn indicate_planar_switch() {
        //outb or outw?
        outb(0x3ce, 0x5);
    }
    fn plane_0() {
        outw(0xc4, 0x102);
    }
    fn plane_1() {
        outw(0x3c4, 0x202);
    }
    fn plane_2() {
        outw(0x3c4, 0x402);
    }
    fn plane_3() {
        outw(0x3c4, 0x802);
    }

    pub fn switch(&mut self, new_planar: usize) {
        //qemu_print("Switching to plane: ");
        //qemu_print_num(new_planar as u64);
        //qemu_print("\n");
        //Error if not 0 <= new_planar <= 4
        if (new_planar > 3) {
            graphics_error();
        }
        Planar::indicate_planar_switch();
        match new_planar {
            0 => Planar::plane_0(),
            1 => Planar::plane_1(),
            2 => Planar::plane_2(),
            3 => Planar::plane_3(),
            _ => panic!("This should not happen!"),
        }
        self.current_planar = new_planar;
    }

    pub fn restore(&mut self) {
        outw(0x3c4, 0xf02);
    }

    pub fn next(&mut self) {
        self.switch((self.current_planar + 1) % 4);
    }
}
