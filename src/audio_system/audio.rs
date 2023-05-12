use crate::qemu_println;
use crate::time;
use crate::tooling::serial::*;

// musical notes using the 12TET system (C, Db, D, ..., Bb, B)
// in the sub-contra octave.
// this is an inefficient way to do it but it gets coded faster
// note that since the frequency (when played) has to be an u32, it will be truncated
// this means that very high-freq notes will sound out of tune
pub struct Notes;

#[allow(non_upper_case_globals)]
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
pub fn note(_note: f64, octave: u8) -> u32 {
    let res = (_note
        * match octave {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 8,
            4 => 16,
            5 => 32,
            6 => 64,
            7 => 128,
            8 => 256,
            _ => panic!(
                "attempted to play note with too high frequency (octave = {})",
                octave
            ),
        } as f64) as u32;
    qemu_println!("{}", res);
    res
}

#[repr(C, packed)]
struct AudioPlayer {
    bits: &'static mut [u8],
    position: usize,
    volume: u8,
}

impl AudioPlayer {
    // play one byte of audio
    fn play_sound(&self) {
        {}
    }
}

// part of (basically all of the useful code) takes "inspiration" from the os dev article on the pc speaker
// this will play the sound forever
pub fn play(frequency: u32) {
    let divisor = 1193180 / frequency;
    let tmp: u8;

    outb(0x43, 0xb6);
    outb(0x42, divisor as u8);
    outb(0x42, (divisor >> 8) as u8);
    //outw(0x42, divisor as u16);
    qemu_println!("{} {} {}", divisor as u8, (divisor >> 8) as u8, divisor);

    tmp = inb(0x61);
    if tmp != (tmp | 3) {
        outb(0x61, tmp | 3);
    }
}

// stop playing sound
pub fn stop() {
    let tmp = inb(0x61) & 0xFC;
    outb(0x61, tmp);
}

// beep() and sweep() should not be used since they just waste cpu time
pub fn beep(freq: u32, duration: u64) {
    play(freq);
    let timer = crate::time::Timer::new(duration, &stop);
}

pub fn sweep(start: i32, end: i32, delay: u64) {
    let step = if start < end { 1.001 } else { 1.0 / 1.001 };
    let mut i = start as f64;
    while i < (end as f64) {
        play(i as u32);
        i *= step;
        time::sleep(delay);
    }
    stop();
}

pub fn pit_test() {}
