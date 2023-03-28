#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod tooling;

use tooling::vga::write_str_at;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    write_str_at("Hello World!", 0, 0, 0xb);
    !panic!("This is a test panic.");
    loop {}
}