use super::font_data;
use super::planar_writer;
use super::planar_writer::VgaPlanarWriter;
use super::surface::Surface;
use super::utils::ColorCode;
use crate::math::vec2::*;

struct FontWriter {
    font: font_data::FontData,
    cursor_pos: Vec2<usize>,

    char_surface_buffer: [[Surface; 16]; 256],
}

impl FontWriter {
    pub fn write(writer: &mut VgaPlanarWriter, s: &str) {}

    pub fn load_char_surface(c: char, color: ColorCode) {}

    pub fn write_char(writer: &mut VgaPlanarWriter) {}
}
