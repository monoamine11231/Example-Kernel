<<<<<<< HEAD
#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod tooling;

use tooling::vga::write_str_at;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write_str_at("Hello World!", 0, 0, 0xb);
    panicking_function();
    loop {}
}

fn panicking_function() -> ! {
    //write_str_at("Panicking function call", 0, 0, 0xb);
    //tooling::panic_handler::stack_trace();
    panic!("This is a test panic.");

    loop {}
}
=======
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
// sudo apt install gcc-multilib (for 32bit files)

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}    
}

        
/* #[no_mangle] means: 
 * use the function or variable name (not its full path) as its symbol name.
 */  

static HELLO: &[u8] = b"Hello from kernel! <3 Unreal mode ";
#[no_mangle]  
#[link_section = ".start"] 
pub extern "C" fn _start() -> ! {  
    let vga_buffer = 0xb8000 as *mut u8;
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
    loop {}
}
>>>>>>> a782506 (Rust linker problem & Problem with PIE)
