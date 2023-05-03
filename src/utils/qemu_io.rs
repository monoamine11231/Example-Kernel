use core::arch::asm;

pub fn qemu_print_byte(b: u8) {
    unsafe {
        asm!("mov dx, {0:x}",
             "mov al, {1}",
             "out dx, al",
             in(reg) 0x3f8 as u16,
             in(reg_byte) b);
    }
}

pub fn qemu_print_nln() {
    unsafe {
        asm!("mov dx, {0:x}",
             "mov al, {1}",
             "out dx, al",
             in(reg) 0x3f8 as u16,
             in(reg_byte) '\n' as u8);
    }
}

pub fn qemu_print_u8(n: u8) {
    for b in n.to_be_bytes() {
        qemu_print_byte(b)
    }
}

pub fn qemu_print_str(s: &str) {
    for b in s.bytes() {
        qemu_print_byte(b);
    }
}

pub fn qemu_println(s: &str) {
    qemu_print_str(s);
    qemu_print_nln();
}
