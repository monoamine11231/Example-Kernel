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

    //How much spacing between character
    horizontal_spacing: Vec2<usize>,
    vertical_spacing: Vec2<usize>,

    //These cursor positions will be retraced by cursor if the writing overextends
    horizontal_retrace: usize,
    vertical_retrace: usize,

    //The writer is confined to writing in a box of these dimensions
    box_size: Vec2<usize>,

    //This flag indicates that the cursor position is not okay for writing
    out_of_bounds_flag: bool,

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

            horizontal_retrace: 0,
            vertical_retrace: 0,

            //Default is non-restricted
            box_size: Vec2::<usize>::new(VgaPlanarWriter::SCAN_LN_CNT, VgaPlanarWriter::COL_CNT),

            out_of_bounds_flag: false,

            char_surface_buffer: [[PLACE_HOLDER_SURFACE; 16]; 128],
            loaded: [[false; 16]; 128],
        };
    }

    //Can we place a character at specified pos? true : yes, false : no
    fn check_horizontal_retrace(&self, pos: Vec2<usize>) -> bool {
        let next_cursor_col = pos.y + (self.font.font_width as usize) + self.horizontal_spacing.y;
        let diff = next_cursor_col - self.horizontal_retrace;

        return !(diff >= self.box_size.y || next_cursor_col >= VgaPlanarWriter::COL_CNT);
    }

    //Vertically speaking, can we place a character at pos? true: yes, false: no
    fn check_vertical_retrace(&self, pos: Vec2<usize>) -> bool {
        let next_cursor_row = pos.x + (self.font.font_height as usize) + self.vertical_spacing.x;
        let diff = next_cursor_row - self.vertical_retrace;

        return !(diff >= self.box_size.x || next_cursor_row >= VgaPlanarWriter::SCAN_LN_CNT);
    }

    //Updates the out_of_bounds flag
    fn check_cursor_pos(&mut self) {
        if (self.cursor_pos.x < self.vertical_retrace
            || self.cursor_pos.y < self.horizontal_retrace
            || self.check_horizontal_retrace(self.cursor_pos) == false
            || self.check_vertical_retrace(self.cursor_pos) == false)
        {
            self.out_of_bounds_flag = true;
        } else {
            self.out_of_bounds_flag = false;
        }
    }

    fn do_horizontal_retrace(&mut self) {
        self.new_line();
        self.cursor_pos.y = self.horizontal_retrace;
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
        for i in 32..127 {
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

    //True if succeeded, otherwise false
    pub fn write_char(&mut self, writer: &mut VgaPlanarWriter, c: char, color: ColorCode) -> bool {
        let c_as_idx = c as usize;
        let color_as_idx = color as usize;

        //Making sure the requested surface has been loaded
        assert!(self.loaded[c_as_idx][color_as_idx] == true);

        if (self.check_horizontal_retrace(self.cursor_pos) == false) {
            self.do_horizontal_retrace();
        }
        if (self.check_vertical_retrace(self.cursor_pos) == false) {
            //Now we can't write, obviously assuming writing goes horizontal left to right
            self.out_of_bounds_flag = true;
            return false;
        }

        let mut surface_to_write = &mut self.char_surface_buffer[c_as_idx][color_as_idx];
        surface_to_write.set_origin(self.cursor_pos);
        writer.write_surface(&surface_to_write);

        self.cursor_pos += Vec2::<usize>::new(0, self.font.font_width as usize);
        self.cursor_pos += self.horizontal_spacing;
        return true;
    }

    pub fn new_line(&mut self) {
        self.cursor_pos += Vec2::<usize>::new(self.font.font_height as usize, 0);
        self.cursor_pos += self.vertical_spacing;
    }

    pub fn write(&mut self, writer: &mut VgaPlanarWriter, s: &str, color: ColorCode) {
        self.horizontal_retrace = self.cursor_pos.y;
        self.vertical_retrace = self.cursor_pos.x;

        for b in s.as_bytes() {
            let byte_as_char = *b as char;
            if (byte_as_char == '\n') {
                self.new_line();
                continue;
            } else if (*b >= 32 && *b <= 126) {
                //If a real character
                self.write_char(writer, byte_as_char, color);
                //If cursor is out of bounds, we should probably stop trying to write characters
                if (self.out_of_bounds_flag) {
                    return;
                };
            } else {
                panic!("Tried to write a wierd character!?")
            }
        }
    }

    //Writes a string, but does not move the cursor
    pub fn write_and_retrace(&mut self, writer: &mut VgaPlanarWriter, s: &str, color: ColorCode) {
        let stashed_cursor_pos = self.cursor_pos;
        self.write(writer, s, color);
        self.set_cursor_pos(stashed_cursor_pos);
    }

    pub fn set_horizontal_spacing(&mut self, spacing: Vec2<usize>) {
        self.horizontal_spacing = spacing;
        self.check_cursor_pos();
    }
    pub fn set_vertical_spacing(&mut self, spacing: Vec2<usize>) {
        self.vertical_spacing = spacing;
        self.check_cursor_pos();
    }

    pub fn set_cursor_pos(&mut self, pos: Vec2<usize>) {
        self.cursor_pos = pos;
        self.horizontal_retrace = pos.y;
        self.vertical_retrace = pos.x;
        self.check_cursor_pos();
    }
    pub fn set_box_size(&mut self, sz: Vec2<usize>) {
        self.box_size = sz;
        self.check_cursor_pos(); //Updating out of bounds flag
    }

    pub fn get_cursor_pos(&mut self) -> Vec2<usize> {
        return self.cursor_pos;
    }
}
