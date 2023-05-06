use crate::tooling::serial::*;

#[repr(C, packed)]
struct AudioPlayer {
    bit_rate: u8,
    position: usize,
    volume: u8,
}

impl AudioPlayer {
    // (this might be haram)
    // play one byte of audio
    #[inline(always)]
    fn play_sound(&self) {
        {}
    }
}

pub fn play_sound(frequency: u32) {
    let Div = 1193180 / frequency;
    let tmp: u8;

    outb(0x43, 0xb6);
    outb(0x42, Div as u8);
    outb(0x42, (Div >> 8) as u8);

    tmp = inb(0x61);
    if tmp != (tmp | 3) {
        outb(0x61, tmp | 3);
    }
}

pub fn no_sound() {
    let tmp = inb(0x61) & 0xFC;
    outb(0x61, tmp);
}

pub fn beep() {
    play_sound(1000);
}