use core::arch::asm;

/// Output a BYTE to a 16-bit serial port address
pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!("mov dx, {0:x}",
             "mov al, {1}",
             "out dx, al",
             in(reg) port,
             in(reg_byte) value,
             options(nostack, preserves_flags, nomem)
        );
    }
}

/// Output a WORD to a 16-bit serial port address
pub fn outw(port: u16, value: u16) {
    unsafe {
        asm!("mov dx, {0:x}",
             "mov ax, {1:x}",
             "out dx, ax",
             in(reg) port,
             in(reg) value,
             options(nostack, preserves_flags, nomem)
        );
    }
}

/// Output a DWORD to a 16-bit serial port address
pub fn outd(port: u16, value: u32) {
    unsafe {
        asm!("mov dx, {0:x}",
             "mov eax, {1:e}",
             "out dx, eax",
             in(reg) port,
             in(reg) value,
             options(nostack, preserves_flags, nomem)
        );
    }
}

/// Returns a BYTE from a 16-bit serial port address
pub fn inb(port: u16) -> u8 {
    let mut value: u8;
    unsafe {
        asm!("mov dx, {0:x}",
             "mov al, {1}",
             "in al, dx",
             in(reg) port,
             out(reg_byte) value,
             options(nostack, preserves_flags, nomem)
        );
    }
    value
}

/// Returns a WORD from a 16-bit serial port address
pub fn inw(port: u16) -> u16 {
    let mut value: u16;
    unsafe {
        asm!("mov dx, {0:x}",
             "mov ax, {1:x}",
             "in ax, dx",
             in(reg) port,
             out(reg) value,
             options(nostack, preserves_flags, nomem)
        );
    }
    value
}

/// Returns a DWORD from a 16-bit serial port address
pub fn ind(port: u16) -> u32 {
    let mut value: u32;
    unsafe {
        asm!("mov dx, {0:x}",
             "mov eax, {1:e}",
             "in eax, dx",
             in(reg) port,
             out(reg) value,
             options(nostack, preserves_flags, nomem)
        );
    }
    value
}
