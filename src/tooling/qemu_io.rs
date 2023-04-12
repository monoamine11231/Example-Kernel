use super::serial::outb;

pub fn qemu_print(string: &str) {
    for b in string.bytes() {
        outb(0x3F8, b);
    }
}

pub fn qemu_println(string: &str) {
    qemu_print(string);
    qemu_print("\n");
}
