#![macro_use]
pub use heapless::String;
use crate::WRITER;
pub const FORMAT_STRING_SIZE: usize = 256; // for now


#[macro_export]
macro_rules! format {
    ($string:expr, $($arg:tt)*) => {{
        use heapless::String;
        use core::fmt::Write;
        use crate::WRITER;
        
        let mut formatted_string = String::<{crate::FORMAT_STRING_SIZE}>::new();
        write!(&mut formatted_string, $string, $($arg)*).unwrap();
        formatted_string
    }};
}

#[macro_export]
macro_rules! qemu_print {
    // only a string literal
    ($string:expr) => {{
        crate::tooling::qemu_io::qemu_print(&$string);
    }};
    // a string literal w/ args
    ($string:expr, $($arg:tt)*) => {{
        use heapless::String;
        use core::fmt::Write;
        use crate::WRITER;
        
        let mut formatted_string = String::<{crate::ORMAT_STRING_SIZE}>::new();
        write!(&mut formatted_string, $string, $($arg)*).unwrap();
        crate::tooling::qemu_io::qemu_print(&formatted_string);
    }};
}

#[macro_export]
macro_rules! qemu_println {
    // no args
    () => {{
        crate::tooling::qemu_io::qemu_print("\n");
    }};
    // only a string literal
    ($string:expr) => {{
        use heapless::String;
        use core::fmt::Write;
        use crate::WRITER;
        
        let mut formatted_string = String::<{crate::FORMAT_STRING_SIZE}>::new();
        write!(&mut formatted_string, $string).unwrap();
        formatted_string.push("\n");
        crate::tooling::qemu_io::qemu_print(&formatted_string);
    }};
    // a string literal w/ args
    ($string:expr, $($arg:tt)*) => {{
        use heapless::String;
        use core::fmt::Write;
        use crate::WRITER;
        
        let mut formatted_string = String::<{crate::FORMAT_STRING_SIZE}>::new();
        write!(&mut formatted_string, $string, $($arg)*).unwrap();
        formatted_string.push('\n');
        crate::tooling::qemu_io::qemu_print(&formatted_string);
    }};
}
#[macro_export]
macro_rules! print {
    // only a string literal
    ($string:expr) => {{
        unsafe {
            WRITER.write_str($string);
        }
    }};
    // a string literal w/ args
    ($string:expr, $($arg:tt)*) => {{
        use heapless::String;
        use core::fmt::Write;
        use crate::WRITER;
        
        let mut formatted_string = String::<{crate::FORMAT_STRING_SIZE}>::new();
        write!(&mut formatted_string, $string, $($arg)*).unwrap();
        unsafe { 
            WRITER.write_str(&formatted_string);
        }
    }};
}

#[macro_export]
macro_rules! println {
    // no args
    () => {{
        unsafe {
            WRITER.newline();
        }
    }};
    // only a string literal
    ($string:expr) => {{
        unsafe {
            use core::fmt::Write;
            use crate::WRITER;
            
            WRITER.write_str($string);
            WRITER.newline();
        }
    }};
    // a string literal w/ args
    ($string:expr, $($arg:tt)*) => {{
        use heapless::String;
        use core::fmt::Write;
        use crate::WRITER;
        
        let mut formatted_string = String::<{crate::FORMAT_STRING_SIZE}>::new();
        write!(&mut formatted_string, $string, $($arg)*).unwrap();
        unsafe { 
            WRITER.write_str(&formatted_string);
            WRITER.newline(); 
        }
    }};
}

pub use print;
pub use println;
pub use qemu_print;
pub use qemu_println;