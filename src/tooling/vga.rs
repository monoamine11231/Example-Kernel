use core::fmt::Write;
/// A custom writer to store a string in a buffer.
pub struct VGAWriter {
    pub buffer: &'static mut [u8],
    pub idx: usize,
}

impl VGAWriter {
    pub fn new() -> Self {
        unsafe {
            let vga_offset = 0xb8000 as *mut u8;
            let vga_buffer_slice = core::slice::from_raw_parts_mut(vga_offset, 4000); // (rows * cols) * (chars + color)
            VGAWriter {
                buffer: vga_buffer_slice,
                idx: 0,
            }
        }
    }
    pub fn get_line(&self) -> usize {
        self.idx / 50
    }
    pub fn get_col(&self) -> usize {
        self.idx % 160
    }
    pub fn copy_to_vga(&self) {
        let vga_buffer = 0xb8000 as *mut u8;
        for i in 0..self.idx {}
    }
    pub fn newline(&mut self) {
        self.idx = (self.idx / 160) * 160;
        self.idx += 160;
    }
    pub fn writeln_color(&mut self, s: &str, color: Option<u8>) -> core::fmt::Result {
        let res = self.write_color(s, color);
        self.newline();
        res
    }
    pub fn write_color(&mut self, s: &str, color: Option<u8>) -> core::fmt::Result {
        let color = match color {
            None => 0xf,
            Some(color) => color,
        };
        for c in s.chars() {
            if self.idx < self.buffer.len() {
                self.buffer[self.idx] = c as u8;
                self.buffer[self.idx + 1] = color;
                self.idx += 2;
            }
        }
        Ok(())
    }
    pub fn write_str_at(&mut self, s: &str, row: usize, col: usize, color: u8) {
        let start_pos = (row * 80 + col) * 2;
        
        for (i, byte) in s.bytes().enumerate() {
            unsafe {
                self.buffer[(start_pos + i * 2) as usize] = byte;
                self.buffer[(start_pos + i * 2 + 1) as usize] = color;
            }
        }
        self.idx = start_pos + s.len();
    }
}
impl Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_color(s, None)
    }
}
pub fn write_str(s: &str, writer: Option<VGAWriter>, color: u8) -> VGAWriter {
    let w = match writer {
        None => VGAWriter::new(),
        Some(writer_arg) => writer_arg,
    };

    return w;
}
pub fn write_str_at(s: &str, row: usize, col: usize, color: u8) {
    let vga_buffer = 0xb8000 as *mut u8;
    let start_pos = (row * 80 + col) * 2;

    for (i, byte) in s.bytes().enumerate() {
        unsafe {
            *vga_buffer.offset((start_pos + i * 2) as isize) = byte;
            *vga_buffer.offset((start_pos + i * 2 + 1) as isize) = color;
        }
    }
}
