#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(strict_provenance)]
#![feature(ptr_from_ref)]
#![feature(abi_x86_interrupt)]
#![allow(unused, unconditional_panic)]

#[macro_use]
extern crate lazy_static;

mod bord;
mod drivers;
mod handlers;
pub mod mem;
mod pic;
mod acpi;
mod tooling;
mod format;
mod test_funcs;
mod audio_system;
mod heap;
mod time;
use core::arch::asm;
use time::Timer;
use core::fmt::Write;
use core::str::Bytes;

use bord::*;
use drivers::ide::IDE;
pub mod fat32;
use heapless::String;
use mem::memory::{self, *};
use tooling::qemu_io::{qemu_print_hex, qemu_println};
use tooling::vga::write_str_at;
use tooling::serial::*;
use core::borrow::BorrowMut;
use audio_system::audio;
use crate::tooling::vga::VGAWriter;
pub const FORMAT_STRING_SIZE: usize = 256;

static mut WRITER: VGAWriter = VGAWriter {
    buffer: &mut [0],
    idx: 0,
    color: 0xf,
};

use crate::handlers::*;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() -> ! {
    unsafe { WRITER = VGAWriter::new(); }
    let mut i = 0;
    qemu_println!("0x40: {:b}\n0x41: {:b}\n0x42: {:b}", inb(0x40), inb(0x41), inb(0x42));

    load_idt(&IDTX);
    pic::init();
    time::init();
    test_funcs::rainbow_print("Hello world!!");
    memory::init();
    // audio::play(audio::note(audio::Notes::A, 4)); // play A4 (440 Hz) (at super high volume -.-)

    let buf: [u8; 10] = [0x10u8; 10];

    let mut ide_processor: IDE = Default::default();
    ide_processor.init();
    let mut fs_processor = fat32::FAT32::new(&mut ide_processor).unwrap();

    let mut buf: [u8; 64] = [0x00u8; 64];
/*
    fs_processor.read_file("KEK/ABA/LOL3.TXT", &mut buf, 420);
    // fs_processor.delete_directory("KEK/ABA").unwrap();
    fs_processor.create_file("KEK", "A.TXT").unwrap();
    fs_processor.create_directory("", "UUU").unwrap();
    fs_processor.create_directory("UUU", "OOO").unwrap();
    fs_processor.create_file("UUU", "B.TXT").unwrap();
    fs_processor.create_file("UUU", "AAA.TXT").unwrap();
    fs_processor.create_file("UUU/OOO", "CD.TXT").unwrap();
    fs_processor.create_file("KEK", "B0.TXT").unwrap();
    let str1: &str = "append from fs wow!";
    fs_processor.write_file("KEK/A.TXT", str1.as_bytes(), str1.len()).unwrap();
    let str2: &str = " [please hope this appends]";
    fs_processor.write_file("LOL.TXT", str2.as_bytes(), str2.len()).unwrap();
    fs_processor.create_file("UUU/OOO", "LOL.TXT").unwrap();
    
    fs_processor.write_file("UUU/OOO/LOL.TXT", str2.as_bytes(), str2.len()).unwrap();
    
    

    /* From reading a file */
    qemu_println(unsafe { core::str::from_utf8_unchecked(&buf) });
*/

    //qemu_print_hex(a);

    // unsafe {
    //     asm!(
    //         "div {0:e}",
    //         in(reg) 0,
    //     )
    // }

    /*let mut my_vector: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
    let mut i = 0u8;
    while true {
        i = i.wrapping_add(1);
        my_vector.push(i);
        qemu_println!("{:?}", &my_vector);
    }
    */
    qemu_println!("0x40: 0x{:X}\n0x41: 0x{:X}\n0x42: 0x{:X}", inb(0x40), inb(0x41), inb(0x42));
    let mut ptr = 0x20 as *mut u128;
    /* while ((ptr as usize) < 0x200) {
        unsafe {*ptr = 0; ptr = ptr.offset(1);}
    } */

    unsafe {
        time::TIMER.init();
    }

    loop {}
}

lazy_static! {
    static ref IDTX: IDT = IDT {
        page_fault: IDTEntry::new(handlers::page_fault, Ring::Zero),
        divide_error: IDTEntry::new(handlers::zero_div, Ring::Zero),
        debug: IDTEntry::new(handlers::debug, Ring::Zero),
        non_maskable_interrupt: IDTEntry::new(handlers::non_maskable_interrupt, Ring::Zero),
        breakpoint: IDTEntry::new(handlers::breakpoint, Ring::Zero),
        overflow: IDTEntry::new(handlers::overflow, Ring::Zero),
        bound_range_exceeded: IDTEntry::new(handlers::bound_range_exceeded, Ring::Zero),
        invalid_opcode: IDTEntry::new(handlers::invalid_opcode, Ring::Zero),
        device_not_available: IDTEntry::new(handlers::device_not_available, Ring::Zero),
        double_fault: IDTEntry::new(handlers::double_fault, Ring::Zero),
        invalid_tss: IDTEntry::new(handlers::invalid_tss, Ring::Zero),
        segment_not_present: IDTEntry::new(handlers::segment_not_present, Ring::Zero),
        stack_segment_fault: IDTEntry::new(handlers::stack_segment_fault, Ring::Zero),
        general_protection_fault: IDTEntry::new(handlers::general_protection_fault, Ring::Zero),
        x87_floating_point: IDTEntry::new(handlers::x87_floating_point, Ring::Zero),
        alignment_check: IDTEntry::new(handlers::alignment_check, Ring::Zero),
        machine_check: IDTEntry::new(handlers::machine_check, Ring::Zero),
        simd_floating_point: IDTEntry::new(handlers::simd_floating_point, Ring::Zero),
        virtualization: IDTEntry::new(handlers::virtualization, Ring::Zero),
        security_exception: IDTEntry::new(handlers::security_exception, Ring::Zero),
        interrupt1: IDTEntry::new(handler1_wtf, Ring::Zero),
        interrupt2: IDTEntry::new(keyboard_handler, Ring::Zero),
        interrupt3: IDTEntry::new(mh3, Ring::Zero),
        interrupt4: IDTEntry::new(mh4, Ring::Zero),
        interrupt5: IDTEntry::new(mh5, Ring::Zero),
        interrupt6: IDTEntry::new(mh6, Ring::Zero),
        interrupt7: IDTEntry::new(mh7, Ring::Zero),
        interrupt8: IDTEntry::new(mh8, Ring::Zero),
        interrupt9: IDTEntry::new(mh9, Ring::Zero),
        interrupt10: IDTEntry::new(mh10, Ring::Zero),
        interrupt11: IDTEntry::new(mh11, Ring::Zero),
        interrupt12: IDTEntry::new(mh12, Ring::Zero),
        interrupt13: IDTEntry::new(mh13, Ring::Zero),
        interrupt14: IDTEntry::new(mh14, Ring::Zero),
        interrupt15: IDTEntry::new(mh15, Ring::Zero),
        interrupt16: IDTEntry::new(mh16, Ring::Zero),
        ..Default::default()
    };
}

fn panicking_function() -> ! {
    //write_str_at("Panicking function call", 0, 0, 0xb);
    //tooling::panic_handler::stack_trace();
    panic!("This is a test panic.");

    loop {}
}

fn waste_time(_time: u64) {
    let mut time = 0;
    while time < _time {
        let waste = VGAWriter::new();
        drop(waste);
        time += 1;
    }
}