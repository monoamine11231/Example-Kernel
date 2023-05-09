use core::arch::asm;

use crate::tooling::serial::{inb, outb};
use crate::qemu_println;

const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;
const PIC_READ_IRR: u16 = 0x0a;
const PIC_READ_ISR: u16 = 0x0b;

const ICW1_ICW4: u8 = 0x01;
const ICW1_SINGLE: u8 = 0x02;
const ICW1_INTERVAL4: u8 = 0x04;
const ICW1_LEVEL: u8 = 0x08;
const ICW1_INIT: u8 = 0x10;

const ICW4_8086: u8 = 0x01;
const ICW4_AUTO: u8 = 0x02;
const ICW4_BUF_SLAVE: u8 = 0x08;
const ICW4_BUF_MASTER: u8 = 0x0C;
const ICW4_SFNM: u8 = 0x10;

const CMD_EOI: u8 = 0x20;

pub fn init() {
    pic_remap(32, 40);

    unsafe {
        asm!("sti");
    }
}

// theft from https://wiki.osdev.org/PIC
fn pic_remap(offset1: u8, offset2: u8) {
    let a = inb(PIC1_DATA);
    let b = inb(PIC2_DATA);
    qemu_println!("pic_a = {}, pic_a = {}", a, b);
    outb(PIC1_CMD, ICW1_INIT | ICW1_ICW4);
    outb(PIC2_CMD, ICW1_INIT | ICW1_ICW4);

    outb(PIC1_DATA, offset1);
    outb(PIC2_DATA, offset2);

    outb(PIC1_DATA, 4);
    outb(PIC2_DATA, 2);

    outb(PIC1_DATA, ICW4_8086);
    outb(PIC2_DATA, ICW4_8086);

    outb(PIC1_DATA, a);
    outb(PIC2_DATA, b);
}

fn pic_get_irq_reg(ocw3: u8) -> u16 {
    outb(PIC1_CMD, ocw3);
    outb(PIC2_CMD, ocw3);
    return ((inb(PIC1_DATA) as u16) << 8) | (inb(PIC2_DATA) as u16);
}
