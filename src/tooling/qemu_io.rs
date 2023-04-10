use core::arch::asm;

pub fn qemu_print(string: &str) {
    for b in string.bytes() {
        unsafe {
            asm!("mov dx, {0:x}",
                 "mov al, {1}",
                 "out dx, al",
                 in(reg) 0x3f8 as u16,
                 in(reg_byte) b);
        }
    }
}

pub fn qemu_println(string: &str) {
    qemu_print(string);
    qemu_print("\n");
}
