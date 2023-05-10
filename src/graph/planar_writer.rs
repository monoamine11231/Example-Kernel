use crate::graph::surface::*;
use crate::graph::utils::*;
use crate::math::vec2::*;
use crate::mem::alloc::kalloc;
use crate::tooling::qemu_io::*;
use crate::tooling::serial::inb;
use crate::tooling::serial::outb;
use crate::tooling::serial::outw;
use crate::tooling::vga::VGAWriter;

use core::cell;
use core::cmp::max;
use core::cmp::min;
use core::ptr::read_volatile;

/*
Represents the write layer of the graphics component
Contains an additional buffer(plane_buffer) that is not the video memory,
thus we essentially get double buffering. This additional buffer is then by its
entirety written to the video memory(present).

https://web.stanford.edu/class/cs140/projects/pintos/specs/freevga/vga/vgamem.htm#intro
http://www.osdever.net/FreeVGA/vga/vgareg.htm
http://www.osdever.net/FreeVGA/vga/colorreg.htm
https://qbmikehawk.neocities.org/articles/palette/

*/

pub struct VgaPlanarWriter {
    video_buffer: &'static mut [u8],
    plane_buffer: [u8; VgaPlanarWriter::PLANE_BUFF_SZ],
    //plane_buffer: &'static mut [u8],
    //plane_buffer: *mut ColorCode,
    pub palette: ColorPalette,
}

impl VgaPlanarWriter {
    //Bit masks for accessing individual bits in each u8 of each bitplane
    const BIT_MAPPING: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 0x8, 0x4, 0x2, 0x1];

    //Resolution stuff
    pub const SCAN_LN_SZ: usize = 80;
    pub const SCAN_LN_CNT: usize = 480;
    pub const COL_CNT: usize = 640;

    //Video memory adress
    const VIDEO_MEM_BASE: *mut u8 = 0xA0000 as *mut u8;

    //Alternative plane buffer memory adress
    const PLANE_BUFF_BASE: *mut u8 = 0x1e8480 as *mut u8;

    //The size of one bitplane(plane)
    const PLANE_SZ: usize = VgaPlanarWriter::SCAN_LN_CNT * VgaPlanarWriter::SCAN_LN_SZ;
    //Size of the video memory, represents one bitplane
    const VIDEO_MEM_SZ: usize = VgaPlanarWriter::PLANE_SZ;
    //The size of this extra buffer, corresponding to the size of 4 bitplanes
    const PLANE_BUFF_SZ: usize = VgaPlanarWriter::PLANE_SZ * 4;

    pub fn new() -> Self {
        //ColorPalette::read_vga_dac_state();

        let mut A = unsafe {
            VgaPlanarWriter {
                video_buffer: core::slice::from_raw_parts_mut(
                    VgaPlanarWriter::VIDEO_MEM_BASE,
                    VgaPlanarWriter::VIDEO_MEM_SZ,
                ),
                /*
                plane_buffer: core::slice::from_raw_parts_mut(
                    VgaPlanarWriter::PLANE_BUFF_BASE,
                    VgaPlanarWriter::PLANE_BUFF_SZ,
                ),
                */
                plane_buffer: [0; VgaPlanarWriter::PLANE_BUFF_SZ],
                /*
                plane_buffer: core::slice::from_raw_parts_mut(
                    p as *mut u8,
                    VgaPlanarWriter::PLANE_BUFF_SZ,
                ),
                */
                palette: ColorPalette::new(),
            }
        };
        VgaPlanarWriter::setup_registers_for_present();
        return A;
    }

    pub fn write_pixel(&mut self, row: usize, col: usize, color: ColorCode) {
        let color_bits: [u8; 4] = [
            (color as u8) & 1,
            (color as u8) & 2,
            (color as u8) & 4,
            (color as u8) & 8,
        ];
        let bit_pos = col % 8;
        let buffer_idx = (VgaPlanarWriter::SCAN_LN_SZ * row + col / 8) as usize;
        let mask = VgaPlanarWriter::BIT_MAPPING[bit_pos as usize];

        for i in 0..4 {
            let bit_value = color_bits[i];
            if (bit_value > 0) {
                self.plane_buffer[buffer_idx + i * VgaPlanarWriter::PLANE_SZ] |= mask;
            } else {
                self.plane_buffer[buffer_idx + i * VgaPlanarWriter::PLANE_SZ] &= (!mask);
            }
        }
    }

    fn setup_registers_for_present() {
        //Data rotate register, no rotation, no modification
        outb(0x3ce, 0x03);
        outb(0x3cf, 0x00);

        //Setting up final bitmask, every bit is taken from the ALU
        outb(0x3ce, 0x08);
        outb(0x3cf, 0xff);

        //Setting "Enable Set/Reset" register to not be used
        outb(0x3ce, 0x01);
        outb(0x3cf, 0x0);
    }

    fn setup_registers_for_reset() {
        //There are faster ways to do this
        VgaPlanarWriter::set_write_mode(0);

        //Setting the "Enable Set/Reset" register to be used on all planes
        outb(0x3ce, 0x01);
        outb(0x3cf, 0xff);

        //Setting the "Set/Reset" register to replicate 00000000 on each plane
        outb(0x3ce, 0x0);
        outb(0x3cf, 0x0);

        //Making sure all planes are enabled
        outb(0xc4, 0x02);
        outb(0xc5, 0xff);
    }

    //Sets which planes are affected by the writes
    fn set_memory_plane_write_enable(mask: u8) {
        outb(0x3c4, 0x02);
        outb(0x3c5, mask);
    }

    fn set_write_mode(mode: u8) {
        assert!(mode < 4);
        let stashed = inb(0x3cf);
        outb(0x3ce, 0x05);
        outb(0x3cf, (stashed & 0xfc) | mode);
    }

    fn col_clip(&mut self, x: i32) -> usize {
        return x.clamp(0, VgaPlanarWriter::COL_CNT as i32 - 1) as usize;
    }
    fn row_clip(&mut self, x: i32) -> usize {
        return x.clamp(0, VgaPlanarWriter::SCAN_LN_CNT as i32 - 1) as usize;
    }

    //mid : (row, col)
    pub fn write_circle(&mut self, mid: (usize, usize), radius: usize, color: ColorCode) {
        let min_x = self.col_clip(mid.1 as i32 - radius as i32);
        let max_x = self.col_clip(mid.1 as i32 + radius as i32);

        let min_y = self.row_clip(mid.0 as i32 - radius as i32);
        let max_y = self.row_clip(mid.0 as i32 + radius as i32);

        for i in min_y..max_y {
            for j in min_x..max_x {
                let d_squared = (mid.0 as i32 - i as i32).pow(2) + (mid.1 as i32 - j as i32).pow(2);
                if (d_squared < (radius as i32).pow(2)) {
                    self.write_pixel(i as usize, j as usize, color);
                }
            }
        }
    }

    //mid : (row, col)
    pub fn write_rect(
        &mut self,
        mid: (usize, usize),
        width: usize,
        height: usize,
        color: ColorCode,
    ) {
        let min_x: usize = self.col_clip(mid.1 as i32 - (width / 2) as i32);
        let max_x: usize = self.col_clip(mid.1 as i32 + ((width + 1) / 2) as i32);

        let min_y: usize = self.row_clip(mid.0 as i32 - (height / 2) as i32);
        let max_y: usize = self.row_clip(mid.0 as i32 + ((height + 1) / 2) as i32);

        for i in min_y..max_y {
            for j in min_x..max_x {
                self.write_pixel(i, j, color);
            }
        }
    }

    //The surface origin depicts the upper left corner of the image
    pub fn write_surface(&mut self, surface: &Surface) {
        let surface_offset = surface.get_origin();
        let should_ignore = surface.get_ignore_color();

        for i in 0..surface.get_height() {
            for j in 0..surface.get_width() {
                let cell_color: ColorCode = surface.get_cell(i, j);
                //write if we are not to ignore this "cell_color", as determined by the ignore_color field of surface
                if (should_ignore.is_some() == true && should_ignore.unwrap() == cell_color) {
                    continue;
                }
                self.write_pixel(i + surface_offset.x, j + surface_offset.y, cell_color);
            }
        }
    }

    //pub fn write_line(dir: Vec2<f32, origin: Vec2, length: f32) {}

    fn reset_plane_buffer(&mut self) {
        for i in 0..VgaPlanarWriter::PLANE_BUFF_SZ {
            self.plane_buffer[i] = 0;
        }
    }

    //Should hopefully reset all 4 planes(untested)
    fn reset_video_memory(&mut self) {
        VgaPlanarWriter::setup_registers_for_reset();
        self.video_buffer.fill(0);
    }

    fn replicate_plane(&mut self, planar: usize) {
        let offset: usize = planar * VgaPlanarWriter::PLANE_SZ;
        for i in 0..VgaPlanarWriter::PLANE_SZ {
            //self.video_buffer[i] = self.plane_buffer[i + offset];
        }
    }

    fn replicate(&mut self) {
        //VgaPlanarWriter::set_write_mode(0);

        for k in 0..4 {
            VgaPlanarWriter::set_memory_plane_write_enable(1 << k);

            unsafe {
                let ptr = self
                    .plane_buffer
                    .as_ptr()
                    .add(k * VgaPlanarWriter::PLANE_SZ);
                self.video_buffer
                    .copy_from_slice(core::slice::from_raw_parts(ptr, VgaPlanarWriter::PLANE_SZ));
            }
        }
    }

    fn on_new_frame(&mut self) {}

    pub fn print_plane(&mut self, planar: usize) {
        let offset = planar * VgaPlanarWriter::PLANE_SZ;
        for i in 0..VgaPlanarWriter::PLANE_SZ {
            qemu_print_num(self.plane_buffer[i + offset] as u64);
        }
    }

    pub fn present(&mut self, frame_nr: u32) {
        qemu_fmt_println("{}", format_args!("Presenting frame: {}", frame_nr));
        self.replicate();
    }

    pub fn fill_screen(&mut self, color: ColorCode) {
        for k in 0..4 {
            let bit = (color as u8) & (1 << k);
            let mut fill_val = if (bit > 0) { 255 } else { 0 };
            unsafe {
                let ptr = self
                    .plane_buffer
                    .as_mut_ptr()
                    .add(k * VgaPlanarWriter::PLANE_SZ);
                let mut slice = core::slice::from_raw_parts_mut(ptr, VgaPlanarWriter::PLANE_SZ);
                slice.fill(fill_val);
            }
        }
    }

    pub fn color_test(&mut self) {
        self.palette
            .update_color_entry(0, CustomColor::new(63, 0, 0));
        qemu_print("TEST!\n");

        self.palette.update_vga_dac_state();
    }
}
