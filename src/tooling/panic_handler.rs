use crate::tooling::vga;
use crate::tooling::vga::VGAWriter;
use core::arch::asm;
use core::fmt::Write;
use core::panic::PanicInfo;
//use core::option;

pub fn format_line_number(line: u32) -> &'static str {
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

#[inline(always)]
pub fn stack_trace_changed(writer: &mut VGAWriter) {
    let mut saved_rbp: u64;
    let mut saved_rsp: u64;

    unsafe {
        asm!(
        "
            mov {into_rbp}, rbp
            mov {into_rsp}, rsp
        ", 
        into_rbp = out(reg) saved_rbp,
        into_rsp = out(reg) saved_rsp
        );
    }

    let mut curr_rbp: u64 = saved_rbp;
    let mut curr_stack_val: u64;
    let mut stack_idx: i32 = 0;
    let maximum_depth = 15;
    let stack_frame_sz = saved_rbp - saved_rsp;

    write!(
        writer,
        "{}",
        format_args!(
            "------Stack trace: RBP = {:#x} & RSP = {:#x} & BYTES: {:#x}",
            saved_rbp, saved_rsp, stack_frame_sz
        )
    )
    .unwrap();
    writer.newline();

    while curr_rbp != saved_rsp && stack_idx < maximum_depth {
        unsafe {
            asm!(
                "
                    mov {stack_record}, [{rbp}]
                ", 
                stack_record = out(reg) curr_stack_val,
                rbp = in(reg) curr_rbp
            );
        }

        write!(
            writer,
            "{}",
            format_args!("STACK_IDX: {:#x} & VALUE: {:#x}", stack_idx, curr_stack_val)
        )
        .unwrap();
        writer.newline();
        stack_idx += 1;
        curr_rbp -= 8;
    }
}

pub fn stack_trace(writer: &mut VGAWriter) {
    let mut ebp: *mut u64;
    let mut saved_ebp: *mut u64;
    let mut saved_rip: u64;
    let mut should_quit: u64;
    /*
       rip: program counter
       rsp: stack pointer
       rbp: frame pointer, snapshot of rsp, constant

    */

    let mut rbx_cpy: u64;

    unsafe {
        loop {
            // ; saved rbp is pointed to by rbp, which is stored in rbx
            // ; rip is 8 bytes above saved rbp
            /*
            asm!("
                mov {0}, [rbx]
                mov rbx, rbp
                cmp rbp,
                je 1f


                1:

                2:



            ", out(reg) rbx_cpy);
            */

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
                 ", out(reg) ebp, out(reg) saved_ebp, out(reg) should_quit, out(reg) saved_rip);
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
                    "EBP = {:#x}, SAVED EBP = {:#x}, CALLER RIP = {:#x} TOP_FRAME = {}",
                    ebp as u64, saved_ebp as u64, saved_rip, should_quit
                )
            )
            .unwrap();
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
pub fn panic(info: &PanicInfo) -> ! {
    let mut vga: VGAWriter = VGAWriter::new();
    vga.write_color("PANIC", Some(0xc)); // Print "PANIC" at row 2, column 0 with color 0xc (light red)

    if let Some(location) = info.location() {
        print_location(&mut vga, location);
    }

    if let Some(message) = info.message() {
        //print_message(*message, None);
    }
    vga.newline();
    vga.newline();
    stack_trace(&mut vga);
    loop {}
}
