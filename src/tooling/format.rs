use heapless::String;

pub const FORMAT_STRING_SIZE: usize = 256; // for now

#[macro_export]
macro_rules! format {
    ($string:expr, $($arg:tt)*) => {{
        let mut formatted_string = String::<{FORMAT_STRING_SIZE}>::new();
        write!(&mut formatted_string, $string, $($arg)*).unwrap();
        formatted_string
    }};
}

#[macro_export]
macro_rules! qemu_print {
    ($string:expr, $($arg:tt)*) => {{
        let mut formatted_string = String::<{FORMAT_STRING_SIZE}>::new();
        write!(&mut formatted_string, $string, $($arg)*).unwrap();
        crate::tooling::qemu_io::qemu_print(&formatted_string);
    }};
}