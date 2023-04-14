use crate::utils::qemu_io;

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

#[derive(PartialEq, Eq, Clone, Copy)]
struct Rect {
    width: u8,
    height: u8,
    mid: (u8, u8),
}

pub fn graphics_error() {
    qemu_io::qemu_println("Oh no!, we got a graphics error!");
}
