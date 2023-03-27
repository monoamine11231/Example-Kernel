use core::fmt::Write;
use core::panic::PanicInfo;
use crate::tooling::vga::write_str_at;

/// A custom writer to store a string in a buffer.
struct BufferWriter {
    buffer: [u8; 64],
    idx: usize,
}

impl Write for BufferWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            if self.idx < self.buffer.len() {
                self.buffer[self.idx] = c as u8;
                self.idx += 1;
            }
        }
        Ok(())
    }
}

fn format_line_number(line: u32) -> &'static str {
    // maximum number of digits possible to print. 
    // currently will not be able to print 5-digit line numbers
    // so make sure not to introduce any bugs on line 10000+ in any files guys...
    const BUFFER_SIZE: usize = 4; 
    static mut BUFFER: [u8; BUFFER_SIZE + 1] = [0; BUFFER_SIZE + 1];

    if line == 0 {
        return "0";
    }

    let mut num = line;
    let mut idx = 0;

    while num > 0 && idx < BUFFER_SIZE {
        let digit = (num % 10) as u8;
        num /= 10;
        unsafe {
            BUFFER[BUFFER_SIZE - 1 - idx] = b'0' + digit;
        }
        idx += 1;
    }

    let start = BUFFER_SIZE - idx;
    unsafe {
        BUFFER[BUFFER_SIZE] = 0;
        let line_str = core::str::from_utf8_unchecked(&BUFFER[start..BUFFER_SIZE]);
        line_str
    }
}

/// Prints the file and line number where the panic occurred.
fn print_location(location: &core::panic::Location) {
    let file = location.file();
    let line = location.line();
    write_str_at(file, 3, 0, 0xc);
    let line_str = format_line_number(line);
    write_str_at("Line:", 4, 0, 0xc);
    write_str_at(&line_str, 4, 6, 0xc);
}

/// Prints the panic message.
fn print_message(message: core::fmt::Arguments) {
    let mut writer = BufferWriter {
        buffer: [0u8; 64],
        idx: 0,
    };
    let _ = write!(&mut writer, "{}", message);
    let message_str = core::str::from_utf8(&writer.buffer[..writer.idx]).unwrap_or("<invalid utf8>");
    write_str_at(message_str, 5, 0, 0xc);
}

/// This function is called on panic.
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    write_str_at("PANIC", 2, 0, 0xc); // Print "PANIC" at row 2, column 0 with color 0xc (light red)

    if let Some(location) = info.location() {
        print_location(location);
    }

    if let Some(message) = info.message() {
        print_message(*message);
    }

    loop {}
}