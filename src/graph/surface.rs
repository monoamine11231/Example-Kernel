use super::font_data::{self, FontData};
use super::utils::ColorCode;
use crate::graph::utils;
use crate::math::vec2::{self, Vec2};
use crate::mem::alloc;
use crate::tooling::qemu_io::*;

/*
Currently the surface size is fixed, this might have to change ones
we get heap allocation.
*/

pub struct Surface {
    width: usize,
    height: usize,
    origin: Vec2<usize>,

    background_color: Option<ColorCode>,
    ignore_color: Option<ColorCode>,

    buffer: *mut ColorCode,
}

impl Surface {
    //If a background color is not given, the font will be given transparent background
    //In which case, the color black will be treated as the transparent color
    pub fn from_font(
        c: char,
        font: FontData,
        color: ColorCode,
        _background_color: Option<ColorCode>,
    ) -> Self {
        let mut new_surface = Surface {
            width: font.font_width as usize,
            height: font.font_height as usize,
            origin: Vec2::<usize>::new(0, 0),

            ignore_color: None,
            background_color: _background_color,

            buffer: alloc::kalloc(12 * 16) as *mut ColorCode, //Should be fine we have set ColorCode rep to be u8?
        };

        if (_background_color.is_none()) {
            new_surface.ignore_color = Some(ColorCode::Black);
        }

        let font_map_idx = font_data::get_font_entry(c);

        //Iterating over every bit in 24 bytes
        for i in 0..24 {
            for j in 0..8 {
                let curr_row = (i * 8 + j) / 12;
                let curr_col = (i * 8 + j) % 12;
                let buffer_idx = curr_row * new_surface.width + curr_col;
                //qemu_fmt_println("{}", format_args!("row: {}, col: {}", curr_row, curr_col));

                if (font_data::FONT_MAPPING[font_map_idx + i] & (128 >> j) > 0) {
                    unsafe { new_surface.buffer.add(buffer_idx).write(color) }
                } else {
                    let bg = _background_color.unwrap_or(ColorCode::Black);
                    unsafe { new_surface.buffer.add(buffer_idx).write(bg) }
                }
            }
        }
        return new_surface;
    }

    pub fn from_buffer(
        p: *mut ColorCode,
        _height: usize,
        _width: usize,
        _ignore_color: Option<ColorCode>,
    ) -> Surface {
        return Surface {
            height: _height,
            width: _width,
            origin: Vec2::<usize>::new(0, 0), //(row, col)
            background_color: Some(ColorCode::Black),
            ignore_color: _ignore_color,
            buffer: p,
        };
    }

    //Create a blank
    pub fn from_blank(_width: usize, _height: usize) -> Surface {
        let new_surface = Surface {
            width: _width,
            height: _height,
            origin: Vec2::<usize>::new(0, 0),
            background_color: Some(ColorCode::Black),
            ignore_color: Some(ColorCode::Black),
            buffer: alloc::kalloc(_width * _height) as *mut ColorCode,
        };
        unsafe {
            new_surface.buffer.write_bytes(0, _width * _height);
        };
        return new_surface;
    }

    #[inline]
    pub fn set_origin(&mut self, new_origin: Vec2<usize>) {
        self.origin = new_origin;
    }

    #[inline]
    pub fn get_cell(&self, row: usize, col: usize) -> ColorCode {
        return unsafe { self.buffer.add(row * self.width + col).read() };
    }

    #[inline]
    pub fn get_ignore_color(&self) -> Option<ColorCode> {
        return self.ignore_color;
    }

    #[inline]
    pub fn get_origin(&self) -> Vec2<usize> {
        return self.origin;
    }
    #[inline]
    pub fn get_width(&self) -> usize {
        return self.width;
    }
    #[inline]
    pub fn get_height(&self) -> usize {
        return self.height;
    }
}
