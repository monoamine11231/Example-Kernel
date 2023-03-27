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
