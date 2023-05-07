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
use core::arch::asm;
use core::fmt::Write;

use bord::*;
use drivers::ide::IDE;
use drivers::pci::pci_device_search_by_class_subclass;
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

use crate::graph::graphics;
use crate::graph::planar_writer;
use crate::graph::surface;
use crate::handlers::*;
use crate::math::vec2;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() -> ! {
    load_idt(&IDTX);
    output_rsp();

    let my_root = math::utils::sqrt(5.0);
    qemu_fmt_println("{}", format_args!("{}", my_root));
    test_graphics_lib();

    output_rsp();
    memory::init();
    pic::init();

    let buf: [u8; 10] = [0x10u8; 10];

    let mut ide_processor: IDE = Default::default();
    ide_processor.init();
    let mut fs_processor = fat32::FAT32::new(&mut ide_processor).unwrap();

    /*
    let mut buf: [u8; 64] = [0x00u8; 64];
    fs_processor.read_file("KEK/ABA/LOL3.TXT", &mut buf, 420);
    fs_processor.delete_directory("KEK/ABA").unwrap();
    fs_processor.create_file("KEK", "A.TXT").unwrap();
    fs_processor.create_directory("", "UUU").unwrap();
    fs_processor.create_directory("UUU", "OOO").unwrap();
    fs_processor.create_file("UUU", "B.TXT").unwrap();
    fs_processor.create_file("UUU", "AAA.TXT").unwrap();
    fs_processor.create_file("UUU/OOO", "CD.TXT").unwrap();
    fs_processor.create_file("KEK", "B0.TXT").unwrap();
    let str1: &str = "append from fs wow!";
    fs_processor
        .write_file("KEK/A.TXT", str1.as_bytes(), str1.len())
        .unwrap();
    let str2: &str = " [please hope this appends]";
    fs_processor
        .write_file("LOL.TXT", str2.as_bytes(), str2.len())
        .unwrap();
    fs_processor.create_file("UUU/OOO", "LOL.TXT").unwrap();

    fs_processor
        .write_file("UUU/OOO/LOL.TXT", str2.as_bytes(), str2.len())
        .unwrap();

    /* From reading a file */
    qemu_println(unsafe { core::str::from_utf8_unchecked(&buf) });

    //qemu_print_hex(a);

    */

    // unsafe {
    //     asm!(
    //         "div {0:e}",
    //         in(reg) 0,
    //     )
    // }

    loop {}
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
    let mut writer = planar_writer::VGA_planar_writer::new();

    //writer.write_pixel_2(0, 0, ColorCode::Blue);

    let mut counter = 0;
    loop {
        if (counter % 2 == 0) {
            writer.fill_screen(ColorCode::Blue);
        } else {
            //writer.fill_screen(ColorCode::Green);
            //writer.write_circle((0, 0), 100, ColorCode::Green);
            writer.fill_screen(ColorCode::Gray);
        }
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
