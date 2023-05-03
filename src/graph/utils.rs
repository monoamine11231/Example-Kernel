use crate::tooling::qemu_io::*;
use crate::tooling::serial::*;
use core::ptr;
//Enum corresponding to the default color palette
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ColorCode {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    White = 0x7,
    Gray = 0x8,
    BrightBlue = 0x9,
    BrightGreen = 0xa,
    BrightCyan = 0xb,
    BrightRed = 0xc,
    BrightMagenta = 0xd,
    Yellow = 0xe,
    BrightWhite = 0xf,
}

//This is the default VGA color palette
const DEFAULT_COLOR_MAPPING: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (0, 0, 42),
    (0, 42, 0),
    (0, 42, 42),
    (42, 0, 0),
    (42, 0, 42),
    (42, 42, 0),
    (42, 42, 42),
    (0, 0, 21),
    (0, 0, 63),
    (0, 42, 21),
    (0, 42, 63),
    (42, 0, 21),
    (42, 0, 63),
    (42, 42, 21),
    (42, 42, 63),
];

//Custom color format, for possibly chagning the color palette
#[derive(Clone, Copy)]
pub struct CustomColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl CustomColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        assert!(r < 64 && g < 64 && b < 64);
        Self {
            red: r,
            green: g,
            blue: b,
        }
    }
}

//Contains the color palatte information, object of planar_writer
pub struct ColorPalette {
    color_entries: [CustomColor; 16],
}

impl ColorPalette {
    pub fn new() -> Self {
        let mut tmp_init = ColorPalette {
            color_entries: [CustomColor {
                red: 0,
                green: 0,
                blue: 0,
            }; 16],
        };
        tmp_init.reset();
        return tmp_init;
    }

    //Reset the palette to the default state
    pub fn reset(&mut self) {
        for i in 0..16 {
            let v = DEFAULT_COLOR_MAPPING[i];
            self.color_entries[i] = CustomColor::new(v.0, v.1, v.2);
        }
    }

    //Update the internal VGA state with the palette object
    pub fn update_vga_dac_state(&self) {
        qemu_print("masking dac\n");
        //outb(0x3c6, 0xff);
        qemu_print("Updating...\n");

        let r = inb(0x3c7);
        if (r == 3) {
            qemu_print("Ready to accept writes\n");
        } else {
            qemu_print("Ready to accept reads\n");
        }

        outb(0x3c8, 0);
        for i in 0..16 {
            //outb(0x3c8, i);
            qemu_print_num(i as u64);
            let r = self.color_entries[i as usize].red;
            let g = self.color_entries[i as usize].green;
            let b = self.color_entries[i as usize].blue;

            outb(0x3c9, r);
            outb(0x3c9, g);
            outb(0x3c9, b);
            /*
            unsafe {
                outb(
                    0x3c9,
                    *((ptr::addr_of!(self.color_entries[i as usize]) as u64 + 8) as *mut u8),
                );
            }
            outb(0x3c9, g);
            outb(0x3c9, b);
            */
        }
    }

    //Write to the color palette
    pub fn update_color_entry(&mut self, entry: u8, color: CustomColor) -> &Self {
        self.color_entries[entry as usize] = color;
        return self;
    }

    //Print out the internal VGA state
    pub fn read_vga_dac_state() {
        qemu_println("------Writing out color entries------");

        for i in 0..16 {
            outb(0x3c7, i);

            let r = inb(0x3c9);
            let g = inb(0x3c9);
            let b = inb(0x3c9);
            qemu_print("");
            qemu_print_hex(r.into());
            qemu_print_hex(g.into());
            qemu_print_hex(b.into());
            qemu_print("\n");
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Rect {
    width: u8,
    height: u8,
    mid: (u8, u8),
}

pub fn graphics_error() {
    qemu_println("Oh no!, we got a graphics error!");
}
