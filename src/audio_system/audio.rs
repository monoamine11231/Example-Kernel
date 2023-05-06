use crate::tooling::serial::*;


// musical notes using the 12TET system (C, Db, D, ..., Bb, B)
// in the sub-contra octave.
// this is an inefficient way to do it but it gets coded faster
// note that since the frequency (when played) has to be an u32, it will be truncated
// this means that very low-freq notes will sound out of tune
//
pub struct Notes;

impl Notes {
    pub const C: f64 = 16.35;
    pub const Db: f64 = 17.32;
    pub const D: f64 = 18.35;
    pub const Eb: f64 = 19.45;
    pub const E: f64 = 20.60;
    pub const F: f64 = 21.83;
    pub const Gb: f64 = 23.12;
    pub const G: f64 = 24.50;
    pub const Ab: f64 = 25.96;
    pub const A: f64 = 27.50;
    pub const Bb: f64 = 29.14;
    pub const B: f64 = 30.87;
}

// frequency ratio between two adjacent notes
pub const SEMITONE_MULTIPLIER: f64 = 1.05946309436;

// inlining this might be a mistake, lets see
#[inline(always)]
fn note(_note: f64, octave: u8) -> u32 {
    (_note * 
    match octave {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        4 => 16,
        5 => 32,
        6 => 64,
        7 => 128,
        8 => 256,
        _ => panic!("attempted to play note with too high frequency (octave = {})", octave)
    } as f64 ) as u32
}

// part of (basically all of the useful code) is inspired by the os dev article on the pc speaker
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

// this will play the sound for 
pub fn play(frequency: u32) {
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

pub fn stop() {
    let tmp = inb(0x61) & 0xFC;
    outb(0x61, tmp);
}

pub fn beep(freq: u32, duration: usize) {
    play(freq);
    // stop();
    qemu_println!("Played sound!");
}


