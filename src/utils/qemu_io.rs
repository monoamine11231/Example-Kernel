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

pub fn qemu_print_num(mut n: u64) {
    if (n == 0) {
        qemu_print_str("0");
        return;
    }

    const BUFFER_SIZE: usize = 100;
    static mut BUFFER: [u8; BUFFER_SIZE + 1] = [0; BUFFER_SIZE + 1];
    let mut counter = 0;

    while (n > 0) {
        let digit = (n % 10) as u8;
        n /= 10;
        unsafe {
            BUFFER[BUFFER_SIZE - counter - 1] = b'0' + digit;
        }
        counter += 1;
    }

    let start = BUFFER_SIZE - counter;
    unsafe {
        BUFFER[BUFFER_SIZE] = 0;
        qemu_print_str(core::str::from_utf8_unchecked(&BUFFER[start..BUFFER_SIZE]));
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
