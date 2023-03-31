
fn set_cr3() {
    
}

pub struct TLB {

}

pub struct PTEFlags {
    flags : u64,
}

impl PTEFlags {
    fn page_present() {
        return flags & 1 << 0;
    }

    fn page_rw(&self) -> bool {
        return (self.flags & (1 << 1)) != 0;
    }

    fn page_user(&self) -> bool {
        return (self.flags & (1 << 2)) != 0;
    }
    // ...
    fn page_accessed(&self) -> bool {
        return (self.flags & (1 << 5)) != 0;
    }

    fn page_dirty(&self) -> bool {
        return (self.flags & (1 << 6)) != 0;
    }

    fn page_pse(&self) -> bool {
        return (self.flags & (1 << 7)) != 0;
    }

    fn page_global(&self) -> bool {
        return (self.flags & (1 << 8)) != 0;
    }
    // ... add the rest
}
// internal 

fn get_page(addr : u64) {
    // verify sign extension
    
    // traverse page table
}

// syscall alloc
pub fn mmap() {
    
}

// internal malloc for the kernel
pub fn kalloc() {

}

