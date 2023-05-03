#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(strict_provenance)]
#![feature(ptr_from_ref)]
#![feature(abi_x86_interrupt)]
#![allow(unused, unconditional_panic)]

#[macro_use]
extern crate lazy_static;

mod acpi;
mod apic;
mod bord;
mod drivers;
mod graph;
mod handlers;
mod math;
pub mod mem;
mod tooling;
mod utils;
use core::arch::asm;
use core::fmt::Write;

use acpi::*;
use apic::MADTX;
use bord::*;
use drivers::pci::pci_device_search_by_class_subclass;
use graph::surface::Surface;
use graph::utils::{ColorCode, CustomColor};
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
use crate::math::vec2;

#[no_mangle]
#[link_section = ".start"]
pub extern "C" fn _start() -> ! {
    load_idt(&IDTX);
    //write_str_at("Hello World!", 0, 0, 0xb);
    output_rsp();
    //qemu_println("hello from serial terminal IO");
    //test_graphics_mode_105_vesa();
    //test_graphics_mode_12();

    let my_root = math::utils::sqrt(5.0);
    qemu_fmt_println("{}", format_args!("{}", my_root));
    test_graphics_lib();
    //test_graphics_mode_12();

    output_rsp();
    //test_graphics_mode_12();
    /*
    unsafe {
        asm!(
            "div {0:e}",
            in(reg) 0,
        )
    }
    */
    //memory::init();
    //apic::init();
    //let (a, b, c) = pci_device_search_by_class_subclass(0x01, 0x01);
    //qemu_print_hex(RSDTX.0.length);
    //qemu_print_hex(a as u32);
    //qemu_print_hex(b as u32);
    //qemu_print_hex(c as u32);

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
            writer.fill_screen(ColorCode::Green);
        }
        counter += 1;
        wait(10000000);
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
        ..Default::default()
    };
}

fn panicking_function() -> ! {
    //write_str_at("Panicking function call", 0, 0, 0xb);
    //tooling::panic_handler::stack_trace();
    panic!("This is a test panic.");

    loop {}
}
