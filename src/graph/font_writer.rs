use core::panic;

use super::font_data;
use super::planar_writer;
use super::planar_writer::VgaPlanarWriter;
use super::surface::Surface;
use super::utils::ColorCode;
use crate::graph::surface::PLACE_HOLDER_SURFACE;
use crate::math::vec2::*;
use crate::output_rsp;
use crate::tooling::qemu_io::qemu_fmt_println;
use crate::tooling::qemu_io::qemu_print;

pub struct FontWriter {
    font: font_data::FontData,
    cursor_pos: Vec2<usize>,

    horizontal_spacing: Vec2<usize>,
    vertical_spacing: Vec2<usize>,

    char_surface_buffer: [[Surface; 16]; 128],
    loaded: [[bool; 16]; 128],
}

impl FontWriter {
    pub fn new(_font: font_data::FontData) -> Self {
        return Self {
            font: _font,
            cursor_pos: Vec2::<usize>::new(0, 0),

            horizontal_spacing: Vec2::<usize>::new(0, 2),
            vertical_spacing: Vec2::<usize>::new(3, 0),

            char_surface_buffer: [[PLACE_HOLDER_SURFACE; 16]; 128],
            loaded: [[false; 16]; 128],
        };
    }

    pub fn load_char_surface(
        &mut self,
        c: char,
        color: ColorCode,
        background_color: Option<ColorCode>,
    ) {
        let c_as_idx = c as usize;
        let color_as_idx = color as usize;
        qemu_print("Loading char surface\n");

        if (self.loaded[c_as_idx][color_as_idx] == true) {
            //Destruct the surface
            //For now there is memory leaks since we do not have any free function
        }
        self.char_surface_buffer[c_as_idx][color_as_idx] =
            Surface::from_font(c, self.font, color, background_color);
        self.loaded[c_as_idx][color_as_idx] = true;
        qemu_print("Successfully loaded char surface!\n");
    }

    pub fn load_text_color(&mut self, color: ColorCode, background_color: Option<ColorCode>) {
        qemu_print("Loading text color...\n");
        for i in 33..127 {
            let c = char::from_u32(i);
            //output_rsp();
            qemu_fmt_println("{}", format_args!("Loading character: {}...", c.unwrap()));
            match c {
                Some(char_to_load) => {
                    self.load_char_surface(char_to_load, color, background_color);
                }
                None => {
                    qemu_print("This should not happen!");
                    panic!("Oh nooo!!")
                }
            };
        }
    }

    pub fn write_char(&mut self, writer: &mut VgaPlanarWriter, c: char, color: ColorCode) {
        let c_as_idx = c as usize;
        let color_as_idx = color as usize;

        //Making sure the requested surface has been loaded
        assert!(self.loaded[c_as_idx][color_as_idx] == true);

        let mut surface_to_write = &mut self.char_surface_buffer[c_as_idx][color_as_idx];
        surface_to_write.set_origin(self.cursor_pos);
        writer.write_surface(&surface_to_write);

        self.cursor_pos += Vec2::<usize>::new(0, self.font.font_width as usize);
        self.cursor_pos += self.horizontal_spacing;
    }

    pub fn new_line(&mut self) {
        self.cursor_pos += Vec2::<usize>::new(self.font.font_height as usize, 0);
        self.cursor_pos += self.vertical_spacing;
    }

    pub fn write(&mut self, writer: &mut VgaPlanarWriter, s: &str, color: ColorCode) {
        for b in s.as_bytes() {
            let byte_as_char = *b as char;
            if (byte_as_char == '\n') {
                self.new_line();
                continue;
            } else if (*b >= 33 && *b <= 126) {
                //If a real character
                self.write_char(writer, byte_as_char, color);
            } else {
                panic!("Tried to write a wierd character!?")
            }
        }
    }

    //Writes a string, but does not move the cursor
    pub fn write_and_retrace(&mut self, writer: &mut VgaPlanarWriter, s: &str, color: ColorCode) {
        let stashed_cursor_pos = self.cursor_pos;
        self.write(writer, s, color);
        self.cursor_pos = stashed_cursor_pos;
    }

    pub fn set_horizontal_spacing(&mut self, spacing: Vec2<usize>) {
        self.horizontal_spacing = spacing;
    }
    pub fn set_vertical_spacing(&mut self, spacing: Vec2<usize>) {
        self.vertical_spacing = spacing;
    }
    pub fn set_cursor_pos(&mut self, pos: Vec2<usize>) {
        self.cursor_pos = pos;
    }
}
