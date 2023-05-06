use crate::tooling::vga;
use crate::tooling::vga::VGAWriter;
use crate::WRITER;
use crate::{print, println, FORMAT_STRING_SIZE, qemu_print, qemu_println};
use heapless::String;
use core::arch::asm;
use core::fmt::Write;
use core::panic::PanicInfo;

//use core::option;

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
fn print_location(writer: &mut VGAWriter, location: &core::panic::Location) {
    let file = location.file();
    let line = location.line();
    writer.write_str_at(file, 3, 0, 0xc);
    let line_str = format_line_number(line);
    writer.write_str_at("Line:", 4, 0, 0xc);
    writer.write_str_at(&line_str, 4, 6, 0xc);
}

/// Prints the panic message.
fn print_message(message: core::fmt::Arguments, writer: Option<VGAWriter>) -> VGAWriter {
    let mut writer: VGAWriter = match writer {
        None => VGAWriter::new(),
        Some(w) => w,
    };
    let _ = write!(writer, "{}", message);
    //let message_str = core::str::from_utf8(&mut writer.buffer[..writer.idx]).unwrap_or("<invalid utf8>");
    //writer.write_str_at(message_str, 5, 0, 0xc);
    return writer;
}

pub fn stack_trace(writer: &mut VGAWriter) {
    let mut rbp: *mut u64;
    let mut saved_rbp: *mut u64;
    let mut saved_rip: u64;
    let mut should_quit: u64;
    //let mut writer : VGAWriter = VGAWriter::new();
    unsafe {
        loop {
            // ; saved rbp is pointed to by rbp, which is stored in rbx
            // ; rip is 8 bytes above saved rbp
            asm!("
                    mov rbx, rbp
                    mov {0}, rbp
                    cmp rbp, 0
                    je 1f
                    mov {1}, [rbx] 
                    mov rbp, [rbx]  
                    sub rbx, 8     
                    mov {3}, [rbx]
                    mov {2}, 1
                    jmp 2f
                    1:
                        mov {2}, 0
                    2:
                ", out(reg) rbp, out(reg) saved_rbp, out(reg) should_quit, out(reg) saved_rip);
            //asm!("
            //    mov {0}, rbp
            //    mov rsp, rbp
            //
            //    pop rbp
            //    mov {1}, rbp
            // ", out(reg) ebp, out(reg) saved_ebp);
            write!(
                writer,
                "{}",
                format_args!(
                    "RBP = {:#x}, SAVED RBP = {:#x}, CALLER RIP = {:#x} TOP_FRAME = {}",
                    rbp as u64, saved_rbp as u64, saved_rip, should_quit
                )
            ); // Some(print_message(format_args!("EBP = {:#x}, SAVED EBP = {:#x}, TOP_FRAME = {}\n", ebp as u64, saved_ebp as u64, should_quit), writer));
            writer.newline();
            if should_quit == 1 {
                break;
            }
        }
    }
}

fn dump_current_frame() {}

/// This function is called on panic.
#[panic_handler]
// i commented old code instead of removing it cuz I am not sure if i broke something while rewriting this
pub fn panic(info: &PanicInfo) -> ! {
    //let mut vga: VGAWriter = VGAWriter::new();
    unsafe {
        WRITER.color = 0xc;
        WRITER.newline();
    }
        qemu_print!("\nPANIC");

        if let Some(message) = info.message() {
            qemu_println!(" - `{}`", message); // message(*message, None);
        } else {
            qemu_println!();
        }

        if let Some(location) = info.location() {
            qemu_print!("at {}", location); //print_location(&mut WRITER, location);
        }

        //unsafe { stack_trace(&mut WRITER) };
    loop {}
}