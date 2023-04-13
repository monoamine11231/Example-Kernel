use core::arch::{asm, x86_64::_MM_FROUND_NO_EXC};

fn set_cr3(mut reg_val : u64) {
    unsafe {
        asm!("mov cr3, {}", in(reg) reg_val);
    }
}


unsafe fn get_cr3() -> u64 {
        let mut reg_val : u64 = 0xf00dbabe;
        asm!("mov {}, cr3", out(reg) reg_val);
        assert!(reg_val != 0xf00dbabe);
        reg_val
}

// four level page translation. 
unsafe fn translate(curr_addr : u64) -> u64 {
    
        let mut curr_addr = curr_addr;
        for i in 0..4 {
            let next_addr = pte_next_entry_offset(*(curr_addr as *mut u64)) as u64 * 8 + (curr_addr);
            curr_addr = next_addr
        }
        curr_addr
}
pub struct Pager {
    phys_base : u64,

}
pub struct PT {
}
pub struct PTE {
    fields : u64
}

pub struct PTEFlags {
    flags: u64,
}

fn pte_next_entry_offset(pte : u64) -> u16 {
    return ((pte >> 12) & 511) as u16; // bits 12 to 12 + 9 determine the addr the next pte
}

impl PTE {


    fn page_present(&self) -> bool {
        return ((self.fields & 1) << 0) != 0;
    }

    fn page_rw(&self) -> bool {
        return (self.fields & (1 << 1)) != 0;
    }

    fn page_user(&self) -> bool {
        return (self.fields & (1 << 2)) != 0;
    }
    
    // ...
    fn page_accessed(&self) -> bool {
        return (self.fields & (1 << 5)) != 0;
    }

    fn page_dirty(&self) -> bool {
        return (self.fields & (1 << 6)) != 0;
    }

    fn page_pse(&self) -> bool {
        return (self.fields & (1 << 7)) != 0;
    }

    fn page_global(&self) -> bool {
        return (self.fields & (1 << 8)) != 0;
    }
    // ... add the rest
}
// internal

fn get_page(addr: u64) {
    // verify sign extension
    // traverse page table
}

// syscall alloc
pub fn mmap() {
    
}

// internal malloc for the kernel
pub fn kalloc() {
    
}
