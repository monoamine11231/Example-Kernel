use crate::graph::utils::*;
use crate::utils::qemu_io::*;
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
        unsafe {
            asm!(
                "
                mov dx, 0x3ce
                mov ax, 0x5
                out dx, ax
            ",
                clobber_abi("efiapi")
            );
        }
    }
    fn plane_0() {
        unsafe {
            asm!(
                "
                mov dx, 0x3c4
                mov ax, 0x102
                out dx, ax
            ",
                clobber_abi("efiapi")
            );
        }
    }
    fn plane_1() {
        unsafe {
            asm!(
                "
                mov dx, 0x3c4
                mov ax, 0x202
                out dx, ax
            "
            );
        }
    }
    fn plane_2() {
        unsafe {
            asm!(
                "
                mov dx, 0x3c4
                mov ax, 0x402
                out dx, ax
            "
            );
        }
    }
    fn plane_3() {
        unsafe {
            asm!(
                "
                mov dx, 0x3c4
                mov ax, 0x802
                out dx, ax
            "
            );
        }
    }

    pub fn switch(&mut self, new_planar: usize) {
        qemu_print_str("Switching to plane: ");
        qemu_print_num(new_planar as u64);
        qemu_print_nln();
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
        unsafe {
            asm!(
                "
                    mov dx, 0x3c4
                    mov ax, 0xf02
                    out dx, ax
                ",
                clobber_abi("efiapi")
            );
        }
    }

    pub fn next(&mut self) {
        self.switch((self.current_planar + 1) % 4);
    }
}
