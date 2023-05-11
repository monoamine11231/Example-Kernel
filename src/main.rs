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
mod graph;
mod handlers;
mod math;
pub mod mem;
mod pic;
mod tooling;
mod utils;
mod format;
mod time;
mod misc;
mod heap;

use core::arch::asm;
use core::fmt::Write;

pub mod input;
use fat32::test_filesystem;
use input::key_codes::KeyPressedCodes;
use input::keyboard::KEYBOARD;

use bord::*;
use drivers::ide::IDE;
use drivers::ac97::AC97;
use graph::font_writer::FontWriter;
use graph::surface::Surface;
use graph::utils::{ColorCode, CustomColor};
pub mod fat32;
use heapless::String;
use math::vec2::Vec2;
use mem::memory::{self, *};
use tooling::qemu_io::{
    qemu_fmt_println, qemu_print, qemu_print_hex, qemu_print_num, qemu_println,
};
use tooling::vga::write_str_at;

use crate::graph::font_data;
use crate::graph::font_writer;
use crate::graph::graphics;
use crate::graph::planar_writer;
use crate::graph::surface;

use crate::handlers::*;
use crate::math::vec2;

pub const FORMAT_STRING_SIZE: usize = 256;

// "options"
const use_fs: bool = false;
const do_graphics_test: bool = true;


#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() -> ! {
    load_idt(&IDTX);

    let my_root = math::utils::sqrt(5.0);
    qemu_fmt_println("{}", format_args!("{}", my_root));

    memory::init();
    pic::init();
    time::init();

    let z = time::Timer::new(1000, &say_hi);
    z.init();

    let mut rng = misc::rand::Rng::new();

    let mut audio_processor: AC97 = AC97::new();
    audio_processor.init().unwrap();

    unsafe {
        // callback 0-4
        KEYBOARD.set_callback0(key_event as fn(i32));
    }

    let buf: [u8; 10] = [0x10u8; 10];

    let mut buf: [u8; 64] = [0x00u8; 64];
    if use_fs {
        let mut ide_processor: IDE = IDE::new();
        ide_processor.init();
        let mut fs_processor = fat32::FAT32::new(&mut ide_processor).unwrap();
        test_filesystem(&mut fs_processor);
    }
    if do_graphics_test {
        test_graphics_lib();
    }

    loop {}
}

pub fn key_event(key: i32) {
    if key == KeyPressedCodes::A as i32 {
        qemu_println("A");
    }
    if key == KeyPressedCodes::B as i32 {
        qemu_println("B");
    }
    if key == KeyPressedCodes::C as i32 {
        qemu_println("C");
    }
    if key == KeyPressedCodes::T as i32 {
        qemu_println!("time since system start: {}.{:03}s",
        time::get_millis() / 1000,
        time::get_millis() % 1000 
    )
    }
    if key == KeyPressedCodes::R as i32 {
        let mut rng = crate::misc::rand::Rng::new();
        qemu_println!("random u32: {:010}", rng.u32())
    }
}

pub fn get_rsp() -> *mut u64 {
    let rsp_pointer: *mut u64;
    unsafe {
        asm!("
        mov {0}, rsp
        ", out(reg) rsp_pointer
        )
    }
    return rsp_pointer;
}

pub fn output_rsp() {
    let rsp = get_rsp();
    qemu_print_num(rsp as u64);
    qemu_print("\n");
}

pub fn wait(t: u64) {
    let mut i = 0;
    let mut counter = 0;
    while (i < t) {
        counter += 1;
        if (i % 2 == 0) {
            i += 1;
        } else {
            i += 2;
        }
    }
}

pub fn test_graphics_lib() {
    qemu_println("Test!");
    let mut writer = planar_writer::VgaPlanarWriter::new();

    let mut font_writer = FontWriter::new(font_data::BASIC_FONT);
    font_writer.load_text_color(ColorCode::Green, Some(ColorCode::Black));
    font_writer.set_cursor_pos(Vec2::<usize>::new(200, 200));
    font_writer.set_box_size(Vec2::<usize>::new(50, 100));
    //font_writer.load_text_color(ColorCode::Green, None);

    //let mut A = Surface::from_font('A', font_data::BASIC_FONT, ColorCode::White, None);
    //A.set_origin(Vec2::<usize>::new(100, 100));

    let mut counter = 0;
    loop {
        if (counter % 2 == 0) {
            writer.fill_screen(ColorCode::Blue);
        } else {
            //writer.fill_screen(ColorCode::Green);
            writer.write_circle((0, 0), 100, ColorCode::Green);
            //writer.fill_screen(ColorCode::Gray);
        }

        font_writer.write_and_retrace(&mut writer, "+++++++++++++++", ColorCode::Green);
        writer.write_line(Vec2::<usize>::new(300,100),Vec2::<usize>::new(304,200) , ColorCode::BrightGreen);


        let cursor_pos = font_writer.get_cursor_pos();
        cursor_pos.print();

        writer.present(counter);
        counter += 1;
        wait(100000000);
    }
    
    //writer.color_test();
    //writer.print_plane(1);
}

pub fn test_graphics_mode_12() {
    unsafe {
        /*
        asm!("
            mov dx, {0:x},
            mov al, {1:x},
            out dx, al
            mov dx, {3:x}
            mov al, {2:x}
            out dx, al
            ",
            in(reg) 0x3ce as u16,
            in(reg) 0x5 as u16,
            in(reg) 0x102 as u16,
            in(reg) 0x3c4 as u16
        );
        */

        asm!(
            "
            mov dx, 0x3ce
            mov ax, 0x5
            out dx, ax
            mov dx, 0x3c4
            mov ax, 0x102
            out dx, ax
            ",
        );

        let mem_pointer: *mut u8 = 0xA0000 as *mut u8;
        let buffer = core::slice::from_raw_parts_mut(mem_pointer, 38400); // (rows * cols) * (chars + color)
        let scan_line_sz = 80;
        let scan_line_cnt = 480;

        for i in 0..scan_line_cnt {
            if (i % 30 == 0) {
                asm!(
                    "
                    mov dx, 0x3ce
                    mov ax, 0x5
                    out dx, ax
                    mov dx, 0x3c4
                    mov ax, 0x202
                    out dx, ax
                    ",
                );
            } else {
                asm!(
                    "
                    mov dx, 0x3ce
                    mov ax, 0x5
                    out dx, ax
                    mov dx, 0x3c4
                    mov ax, 0x102
                    out dx, ax
                    ",
                );
            }

            for j in 0..scan_line_sz {
                buffer[i * scan_line_sz + j] = 7;
            }
        }
        asm!(
            "
            mov dx, 0x3ce
            mov ax, 0x5
            out dx, ax
            mov dx, 0x3c4
            mov ax, 0x202
            out dx, ax
            ",
        );

        for i in 0..scan_line_cnt {
            for j in 0..scan_line_sz {
                qemu_print_num(buffer[i * scan_line_sz + j] as u64);
                if (i % 30 == 0) {
                    //qemu_print_num(buffer[i * scan_line_sz + j] as i32);
                    //buffer[i * scan_line_sz + j] = buffer[i * scan_line_sz + j] & 1
                }
            }
        }
    }
}

//Packed memory layout
pub fn test_graphics_mode_105_vesa() {
    unsafe {
        let page_size = 65536; //64kb
        let scan_line_cnt = 768;
        let scan_line_sz = 1024;
        let mem_pointer: *mut u8 = 0xA0000 as *mut u8;
        let buffer = core::slice::from_raw_parts_mut(mem_pointer, 786432); // (rows * cols) * (chars + color)
                                                                           /*
                                                                           for i in 0..12 {
                                                                               asm!("
                                                                                   mov ax, 0x4f05
                                                                                   mov bh, 0

                                                                               ")
                                                                               for j in 0..page_size {

                                                                               }
                                                                           }
                                                                           */

        for i in 0..786432 {
            buffer[i] = 1;
        }
        /*
        for i in 0..scan_line_cnt {
            for j in 0..scan_line_sz {
                buffer[i * scan_line_cnt + j] = 1;
            }
        }
        */
    }
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

fn say_hi() {
    qemu_println!("Hi!");
}