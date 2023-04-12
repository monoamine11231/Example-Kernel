use core::arch::asm;

/* Output byte to 16 bit port */
pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!("mov dx, {0:x}",
             "mov al, {1}",
             "out dx, al",
             in(reg) port,
             in(reg_byte) value
        );
    }
}

/* Output word to 16 bit port */
pub fn outw(port: u16, value: u16) {
    unsafe {
        asm!("mov dx, {0:x}",
             "mov ax, {1:x}",
             "out dx, ax",
             in(reg) port,
             in(reg) value
        );
    }
}

/* Output dword to 16 bit port */
pub fn outd(port: u16, value: u32) {
    unsafe {
        asm!("mov dx, {0:x}",
             "mov eax, {1:e}",
             "out dx, eax",
             in(reg) port,
             in(reg) value
        );
    }
}

/* Input byte from 16 bit port */
pub fn inb(port: u16) -> u8 {
    let mut value: u8;
    unsafe {
        asm!("mov dx, {0:x}",
             "mov al, {1}",
             "in al, dx",
             in(reg) port,
             out(reg_byte) value

        );
    }
    return value;
}

/* Input word from 16 bit port */
pub fn inw(port: u16) -> u16 {
    let mut value: u16;
    unsafe {
        asm!("mov dx, {0:x}",
             "mov ax, {1:x}",
             "in ax, dx",
             in(reg) port,
             out(reg) value

        );
    }
    return value;
}

/* Input dword from 16 bit port */
pub fn ind(port: u16) -> u32 {
    let mut value: u32;
    unsafe {
        asm!("mov dx, {0:x}",
             "mov eax, {1:e}",
             "in eax, dx",
             in(reg) port,
             out(reg) value

        );
    }
    return value;
}
