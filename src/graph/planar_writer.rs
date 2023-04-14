use crate::graph::planar::*;
use crate::graph::utils::*;
use crate::utils::qemu_io::*;

use core::cmp::max;
use core::cmp::min;

pub struct VGA_planar_writer {
    video_buffer: &'static mut [u8],
    plane_buffer: &'static mut [u8],
    plane: Planar,
}

impl VGA_planar_writer {
    const BIT_MAPPING: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 0x8, 0x4, 0x2, 0x1];

    const SCAN_LN_SZ: usize = 80;
    const SCAN_LN_CNT: usize = 480;
    const COL_CNT: usize = 640;

    const VIDEO_MEM_BASE: *mut u8 = 0xA0000 as *mut u8;
    const VIDEO_MEM_SZ: usize = VGA_planar_writer::SCAN_LN_CNT * VGA_planar_writer::SCAN_LN_SZ;
    const VIDEO_MEM_EXT_BASE: *mut u8 = 0x1e8480 as *mut u8;
    const VIDEO_MEM_EXT_SZ: usize =
        VGA_planar_writer::SCAN_LN_CNT * VGA_planar_writer::SCAN_LN_SZ * 4;

    const PLANE_SZ: usize = VGA_planar_writer::SCAN_LN_CNT * VGA_planar_writer::SCAN_LN_SZ;

    pub fn new() -> Self {
        let mut A = unsafe {
            VGA_planar_writer {
                video_buffer: core::slice::from_raw_parts_mut(
                    VGA_planar_writer::VIDEO_MEM_BASE,
                    VGA_planar_writer::VIDEO_MEM_SZ,
                ),
                plane_buffer: core::slice::from_raw_parts_mut(
                    VGA_planar_writer::VIDEO_MEM_EXT_BASE,
                    VGA_planar_writer::VIDEO_MEM_EXT_SZ,
                ),
                plane: Planar::new(),
            }
        };
        A.reset_ext_memory();
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
        let buffer_idx = (VGA_planar_writer::SCAN_LN_SZ * row + col / 8) as usize;
        let mask = VGA_planar_writer::BIT_MAPPING[bit_pos as usize];

        for i in 0..4 {
            let bit_value = color_bits[i];
            if (bit_value > 0) {
                self.plane_buffer[buffer_idx + i * VGA_planar_writer::PLANE_SZ] |= mask;
            } else {
                self.plane_buffer[buffer_idx + i * VGA_planar_writer::PLANE_SZ] &= (!mask);
            }
        }
    }

    fn col_clip(&mut self, x: i32) -> usize {
        return x.clamp(0, VGA_planar_writer::COL_CNT as i32 - 1) as usize;
    }
    fn row_clip(&mut self, x: i32) -> usize {
        return x.clamp(0, VGA_planar_writer::SCAN_LN_CNT as i32 - 1) as usize;
    }

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

    fn reset_ext_memory(&mut self) {
        for i in 0..VGA_planar_writer::VIDEO_MEM_EXT_SZ {
            self.plane_buffer[i] = 0;
        }
    }

    fn reset_video_memory(&mut self) {
        for i in 0..VGA_planar_writer::VIDEO_MEM_SZ {
            self.video_buffer[i] = 0;
        }
    }

    fn replicate_plane(&mut self, planar: usize) {
        let offset: usize = planar * VGA_planar_writer::PLANE_SZ;
        for i in 0..VGA_planar_writer::PLANE_SZ {
            self.video_buffer[i] = self.plane_buffer[i + offset];
        }
    }

    pub fn print_plane(&mut self, planar: usize) {
        let offset = planar * VGA_planar_writer::PLANE_SZ;
        for i in 0..VGA_planar_writer::PLANE_SZ {
            qemu_print_num(self.plane_buffer[i + offset] as u64);
        }
    }

    pub fn present(&mut self) {
        qemu_println("Presenting new frame");
        for i in 0..4 {
            self.plane.switch(i);
            self.replicate_plane(i);
        }
        //self.plane.restore();
    }

    pub fn clear_screen(&mut self) {
        self.reset_ext_memory();
    }

    pub fn restore(&mut self) {
        self.plane.restore();
    }
}
