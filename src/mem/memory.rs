use core::arch::{asm};
use core::fmt::Display;
use core::mem::size_of;
use core::ptr;
use crate::tooling::qemu_io::qemu_fmt_println;
use crate::tooling::qemu_io::qemu_println;

use core::fmt::Arguments;

pub fn set_cr3(mut reg_val : u64) {
    unsafe {
        asm!("mov cr3, {}", in(reg) reg_val);
    }
}


pub unsafe fn get_cr3() -> u64 {
        let mut reg_val : u64 = 0xf00dbabe;
        asm!("mov {}, cr3", out(reg) reg_val);
        assert!(reg_val != 0xf00dbabe);
        reg_val
}
pub fn set_cr4(mut reg_val : u64) {
    unsafe {
        asm!("mov cr4, {}", in(reg) reg_val);
    }
}


pub unsafe fn get_cr4() -> u64 {
        let mut reg_val : u64 = 0xf00dbabe;
        asm!("mov {}, cr4", out(reg) reg_val);
        assert!(reg_val != 0xf00dbabe);
        reg_val
}
pub struct AddrSpace {
    pub phys_base : u64,
    pub pml4 : u64,
}

pub fn init() -> AddrSpace {
    unsafe {
        let mut aspace = AddrSpace {
            phys_base : 0,
            pml4 : get_cr3(),
        };
        let new_map = aspace.create_setup_mapping();
        
        set_cr3(new_map as u64);
        let cr3 = get_cr3();
        assert!(cr3 == 0x200 * 0x1000);

        aspace = AddrSpace {
            phys_base : 0,
            pml4 : cr3,
        };
        let mut pt = PT::from_cr3();
        //let taddr = ((1 as u64) << (30)) + (511 * 512 * 0x1000) as u64;
        //let trace = aspace.translate_trace(taddr);
        
        aspace
    }

}


// chungus code will be refactored. It looks like this because I had to root out a bug
impl AddrSpace {
    // four level page translation. Call a closure f at each level of translation
    unsafe fn translate_and_process(&self, addr : u64, f : unsafe fn(&mut PT, u64)) -> u64 {
        
        // This will technically not be PTE during initialization, but this is fine
        let mut curr_pte : &PTE = &*((self.phys_base + self.pml4 + addr & (512 << 39)) as *mut PTE);

        let mut i = 3;
        
        loop {
            // do things with the table before advancing to the next level
            let mut curr_table = &mut *((ptr::addr_of!(curr_pte) as u64 & 0x1000) as *mut PT);
            f(curr_table, i);

            // I will fix this
            if i == 0 {
                break;
            }
            i -= 1;

            // update variables to reflect having advanced to the next table
            curr_pte = curr_pte.get_next_pte(self.phys_base, i);
        }
        ptr::addr_of!(curr_pte) as u64
    }

    // traverse pml4 to a given depth
    unsafe fn translate_to_depth(&self, addr : u64, depth : u64) -> u64 {
        // This will technically not be PTE during initialization, but this is fine
        let mut curr_pte : &PTE = &*((self.phys_base + self.pml4 + addr & (512 << 39)) as *mut PTE);
        let mut i = 3;
        
        loop {
            if (3 - i) > depth {
                break;
            }
            i -= 1;
        
            // update variables to reflect having advanced to the next table
            curr_pte = curr_pte.get_next_pte(self.phys_base, i);
        }
        ptr::addr_of!(curr_pte) as u64
    }

    // return the entire chain of translation
    pub unsafe fn translate_trace(&self, addr : u64) -> [*const PTE; 4] {
        // This will technically not be PTE during initialization, but this is fine
        let mut curr_pte : &mut PTE = &mut *((self.phys_base + self.pml4) as *mut PTE);
        let mut i = 3;
        let mut ptes : [*const PTE; 4] = [core::ptr::null(), core::ptr::null(), core::ptr::null(), core::ptr::null()];
        loop {
            

            // update variables to reflect having advanced to the next table
            qemu_fmt_println("{}", format_args!("curr {:x}",  ptr::addr_of_mut!(*curr_pte) as u64));

            let next_addr : u64 = (curr_pte.next_table_addr() + curr_pte.extract_offset(addr, i) * 8);
            qemu_fmt_println("{}", format_args!("PT idx {}, Next addr {:#x}, Next aligned {:#x}, Next offset {}", i, next_addr, curr_pte.next_table_addr(), curr_pte.extract_offset(addr, i)));

            //qemu_fmt_println("{}", format_args!("Next {:x}", *(next_addr as *mut u64)));

            ptes[i as usize] = curr_pte;
            curr_pte =  &mut *(next_addr as *mut PTE);

            
            
            if i == 0 {
                break;
            }
            i -= 1;

        }
        qemu_println("trace completed");

        ptes
    }
    // FIXME hardcoded for the bootloader offset
    // Assumes identity mapping
    pub unsafe fn create_setup_mapping(&mut self) -> *const PT {
        // offset found in bootloader/paging.s
        let mut PDPT = &mut *(0x71000 as *mut PT);
        let relocated_pml4_addr : u64 = (0x200 * 0x1000);

        kmemcpy(0x70000 as *const u8, relocated_pml4_addr as *mut u8, 0x1000 * 5);

        let mut new_pml4 = (relocated_pml4_addr as *mut PT);
        
        let mut paging_offset = 0x0 + 0x200 * 0x1000 + (0x1000 * 5); // phys_base must be 0. Start the paging after the first 2 MiB, we are aiming for 4 MiB so we can map 1 GiB
        //qemu_fmt_println("\n*pdpt {:x}", format_args!("c {:#x} \n", (&mut *new_pml4).entries[1]));
        let mut new_pdpt = PDPT;// &mut *((relocated_pml4_addr + 0x1000) as *mut PT);
       // (&mut *new_pml4).entries[0] = paging_offset + 3;// 3 = page_rw | page_present

        let mut new_pdt = PT::new_at(paging_offset);
        new_pdpt.entries[1] = paging_offset + 3;// 3 = page_rw | page_present
        //*(0x71008 as *mut u64) = *(*(0x71000 as *mut u64) as *mut u64);
        //qemu_fmt_println("pdpt 0 {}", format_args!("a {:x}\n", (new_pdpt).entries[0]));

        //qemu_fmt_println("", format_args!("pdpt addr of entry 1 {:#x}\n", (ptr::addr_of!(new_pdpt.entries[1]) as u64)));

        //qemu_fmt_println("pdpt 1 {}", format_args!("pdpt 1 {:#x}\n", ((new_pdpt.entries[1]))));

        paging_offset += 0x1000;
        

        let mut i = 0;
        
            let mut j = 0;
            while j < 400 {
                let mut new_pt = &mut *(paging_offset as *mut PT); 
                new_pdt.entries[j] = paging_offset + 3; // 3 = page_rw | page_present
                paging_offset += 0x1000;

                let mut k = 0;
                while k < 512 {
                    // + paging_offset + 0xFFFFFFFFFF
                    new_pt.entries[k] = 3 + ((((i<<21) + (j << 12) + k * 0x1000) as u64) + (0x600 * 0x1000) as u64); // 3 = page_rw | page_present
                    
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        
       //qemu_fmt_println("paging_offset   {}", format_args!("pdt:   {:#x}", (&mut *(71008 as *mut PTE)).next_table_addr()));

        let highest_address = 0b111111111111 + (0b111111111 << 12) + (0x400 << (12 + 9)) + (1 << (12 + 9 + 9));
        qemu_fmt_println("paging_offset   {}", format_args!("virt base:   {:#x}", 0x40000000));
        qemu_fmt_println("highest_address {}", format_args!("virt length: {:#x}", 512 * 512 * 4096));

        new_pml4 as *const PT

    }
    
 
}

// 2 MiB alignment 2097152
#[repr(C, align(4096))]
pub struct PT {
    pub entries : [u64 ; 512]
}

// FIXME: we are assuming an identity mapping by the bootloader
impl PT {
    pub fn map_page(&mut self, addr : u64, page_cnt : u64) {

    }

    pub fn copy_from(to_copy : &PT, target_addr : *mut u8) -> &'static mut Self {
        let pt : *const PT = ptr::from_ref(to_copy);
        unsafe {
            kmemcpy(pt as *const u8, target_addr, 512 * 8);
            return &mut *(target_addr as *mut PT);
        }

    }

    // alias a given PTE. Useful for making high regions mirror low regions, etc
    pub fn alias_self(&mut self, from : usize, to : usize) {
        self.entries[to] = self.entries[from];
    }
    pub unsafe fn new() -> Self {
        PT {
            entries : [0u64; 512]
        }
    }
    // dereference given pointer and return a (safer) borrow to a zeroed page table
    pub unsafe fn new_at(addr : u64) -> &'static mut Self {
        let pt =  &mut *(addr as *mut PT);
        let mut i = 0;
        while i < 512 {
            pt.entries[i] = 0;
            i += 1;
        }
        pt
    }

   
    // dereference pointer in cr3 and return a (safer) borrow to the page table
    pub unsafe fn from_cr3() -> &'static mut Self {
        &mut *(get_cr3() as *mut PT)
    }
}
pub struct PTE(u64);

impl PTE {
    pub fn new(pte : u64) -> PTE {
        Self(pte)
    }
    // given a table level and a phys_base, get the entry pointed to by this pte
    pub unsafe fn get_next_pte(&self, phys_base : u64, curr_table_level : u64) -> &'static mut Self {
        let next_addr = phys_base; // + self.next_entry_addr() + self.next_entry_offset(curr_table_level) * 8;
        panic!("this should not be used");
        return &mut *(next_addr as *mut PTE)
    }

    pub fn next_table_addr(&self) -> u64 {
        // remove bottom 12 bits (one page = 4kib = 12 bits) and remove the flags above the 48th bit
        return self.0 & (!(((1 << 12) - 1 ) as u64)); // bits 12 to 12 + 9 determine the addr the next pte
    }

    pub fn extract_offset(&self, addr : u64, table_idx : u64) -> u64 {
        return ((addr) & (511 << (12 + table_idx * 9))) >> (12 + table_idx * 9); // bits 12 to 12 + 9 determine the addr the next pte
    }

    fn page_present(&self) -> bool {
        return ((self.0 & 1) << 0) != 0;
    }

    fn page_rw(&self) -> bool {
        return (self.0 & (1 << 1)) != 0;
    }

    fn page_user(&self) -> bool {
        return (self.0 & (1 << 2)) != 0;
    }
    
    // ...
    fn page_accessed(&self) -> bool {
        return (self.0 & (1 << 5)) != 0;
    }

    fn page_dirty(&self) -> bool {
        return (self.0 & (1 << 6)) != 0;
    }

    fn page_pse(&self) -> bool {
        return (self.0 & (1 << 7)) != 0;
    }

    fn page_global(&self) -> bool {
        return (self.0 & (1 << 8)) != 0;
    }
    // ... add the rest
}

impl core::fmt::Display for PTE {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "\npte value : {:0x} \n", self.0);

        write!(f, "next addr : {:0x} \n", self.next_table_addr());

        write!(f, "present   : {} \n", self.page_present());
        write!(f, "rw        : {} \n", self.page_rw());
        write!(f, "accessed  : {} \n", self.page_accessed());
        write!(f, "dirty     : {} \n", self.page_dirty());
        write!(f, "PSE       : {} \n", self.page_pse());
        write!(f, "global    : {} \n", self.page_global());
        Ok(())
    }
}
impl core::fmt::Display for PT {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut i = 0;
        // ranges ( 0..512 ) are iterators, so a while is used in its place
        while i < 512 { 
            
            let mut entry = (ptr::addr_of!(self) as u64 + (i * 8)) as *mut PTE;
            
            unsafe {
                write!(f, "{}\n\n", *entry);
            }
            i+=1;
        }
      
        Ok(())
    }
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

// todo : use c_void instead of u64
// todo : inline asm implementation using movsb
// copies sz bytes, from -> to
pub fn kmemcpy(from : *const u8, to : *mut u8, sz : usize) {
     let mut i : usize = 0;
     // because ranges are iterators, we use while loops instead to avoid memory issues
     while i < sz {
        unsafe {
            *((to as usize + i) as *mut u8) = *((from as usize + i) as *const u8)
        }
        i += 1;
    }
}
pub fn kmemset(str : *const u8, c : u8, n : usize) {
    let mut i : usize = 0;
    // because ranges are iterators, we use while loops instead to avoid memory issues
    while i < n {
       unsafe {
           *((str as usize + i) as *mut u8) = c
       }
       i += 1;
   }
}