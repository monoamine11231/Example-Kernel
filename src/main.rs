#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

mod tooling; // Declare the debug module
use tooling::vga::write_str_at;

// cargo build --target x86_64-unknown-none


/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


#[no_mangle]
pub extern "C" fn _start() -> ! {
    write_str_at("Hello World!", 0, 0, 0xb);

    loop {}
}