const RAINBOW_COLORS: [u8; 7] = [4, 6, 14, 2, 1, 3, 5];
pub fn rainbow_print(string: &str) {
    let mut i = 0;
    for character in string.as_bytes() {
        unsafe {
            crate::WRITER.color = RAINBOW_COLORS[i % 7];
        }
        i += 1;
        print!("{}", char::from(*character));
    }
}