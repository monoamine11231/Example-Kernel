use core::mem::size_of;

use core::{arch::asm, panic};

#[repr(u8)]
pub enum Ring {
    Zero = 0,
    One,
    Two,
    Three,
}

#[repr(C, packed(1))]
#[derive(Default, Debug)]

pub struct SegmentDescriptor {
    pub segment_limit: u16,
    pub base_addr: u16,
    pub base: u8,
    pub misc: u16,
    pub base2: u8,
}

impl SegmentDescriptor {
    fn new(func: &dyn FnMut() -> ()) -> Self {
        let mut x = Self::default();

        x.misc |= 1 << 7; // P flag
        x
    }
}

pub fn load_idt(idt: &'static IDT) {
    let x = IDTDescriptor {
        offset: idt,
        size: size_of::<IDT>() as u16 - 1,
    };

    unsafe {
        asm!(
            "lidt [{x}]",
            // "sti", // double fault when we enable interrupts, missing handlers, fix later
            x = in(reg) &x,
            options(nostack)
        );
    }
}

#[repr(C, packed(1))]
struct IDTDescriptor<'a> {
    size: u16,
    offset: &'a IDT,
}

#[repr(C)]
#[derive(Default)]
pub struct IDT {
    pub divide_error: IDTEntry,
    pub debug: IDTEntry,
    pub non_maskable_interrupt: IDTEntry,
    pub breakpoint: IDTEntry,
    pub overflow: IDTEntry,
    pub bound_range_exceeded: IDTEntry,
    pub invalid_opcode: IDTEntry,
    pub device_not_available: IDTEntry,
    pub double_fault: IDTEntry,
    pub reserved_9: IDTEntry,
    pub invalid_tss: IDTEntry,
    pub segment_not_present: IDTEntry,
    pub stack_segment_fault: IDTEntry,
    pub general_protection_fault: IDTEntry,
    pub page_fault: IDTEntry,
    pub reserved_15: IDTEntry,
    pub x87_floating_point: IDTEntry,
    pub alignment_check: IDTEntry,
    pub machine_check: IDTEntry,
    pub simd_floating_point: IDTEntry,
    pub virtualization: IDTEntry,
    pub security_exception: IDTEntry,
    pub more_reserved: [IDTEntry; 10],
}

type HandlerFunc = extern "x86-interrupt" fn();

#[repr(C, packed(1))]
pub struct IDTEntry {
    offset1: u16,
    segment_selector: SegmentSelector,
    ist: u8,
    attribs: u8,
    offset2: u16,
    offset3: u32,
    reserved: u32,
}

impl IDTEntry {
    pub fn new(f: HandlerFunc, dpl: Ring) -> Self {
        let handler_addr: u64 = f as u64;
        let mut attribs = 1 << 7; // P flag
        attribs |= (dpl as u8) << 5;
        attribs |= 0b1110; // type = interrupt gate
        let ist = 0; // Interrupt Stack Table?

        Self {
            offset1: handler_addr as u16,
            offset2: (handler_addr >> 16) as u16,
            offset3: (handler_addr >> 32) as u32,
            ist,
            attribs,
            reserved: 0,
            segment_selector: get_cs(),
        }
    }
}
impl Default for IDTEntry {
    fn default() -> Self {
        Self {
            attribs: 0b1110,
            ist: 0,
            offset1: 0,
            segment_selector: SegmentSelector(0),
            reserved: 0,
            offset2: 0,
            offset3: 0,
        }
    }
}

// https://wiki.osdev.org/Segment_Selector
struct SegmentSelector(u16);

impl SegmentSelector {
    fn new(idx: u16, is_gdt: bool, ring: Ring) -> Self {
        let mut x: Self = SegmentSelector(idx << 3);
        if is_gdt {
            x.0 |= 1 << 2;
        }
        x.0 |= ring as u16;
        x
    }
}

#[repr(C, packed(1))]
#[derive(Default)]
pub struct GDTR {
    pub limit: u16,
    pub base: u64,
}

impl GDTR {
    pub fn get_segment(&self, idx: u16) -> &'static SegmentDescriptor {
        if idx >= self.limit {
            panic!("gdt segment idx exeded limit");
        }
        let x: *const SegmentDescriptor =
            core::ptr::from_exposed_addr((self.base + 8 * idx as u64) as usize);

        unsafe { &*x }
    }
}

pub fn store_gdt() -> GDTR {
    let mut p = GDTR::default();
    unsafe {
        asm!(
            "sgdt [{x}]",
            x = in(reg) &mut p,
            options(nostack, preserves_flags)
        );
    }
    p
}

fn get_cs() -> SegmentSelector {
    let mut x: u16;
    unsafe {
        asm!("mov {0:x}, cs", out(reg) x, options(nostack)) // fler options
    }

    SegmentSelector(x)
}
