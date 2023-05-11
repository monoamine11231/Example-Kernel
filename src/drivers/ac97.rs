use crate::tooling::qemu_io::{qemu_println, qemu_print_hex};
use crate::tooling::serial::*;
use crate::qemu_print;
use super::pci::{pci_device_search_by_class_subclass, pci_get_header_type, pci_get_header_0x00, pci_get_bar_address, pci_get_header_0x01, pci_read_u8, pci_write_u8};
use crate::misc::rand;
use crate::format;
/*  
* Shameless theft from https://wiki.osdev.org/AC97
*/

/* These are the offsets (relative to BAR0) of the Native Audio Mixer registers */
pub const OFFSET_RESET: u16         = 0x00;
pub const OFFSET_VOLUME: u16        = 0x02;
pub const OFFSET_PCM_VOLUME: u16    = 0x18;

/* These are the offsets (relative to BAR1) of the Native Audio Bus Master registers */
pub const OFFSET_PCM_IN: u16        = 0x00; // Register box
pub const OFFSET_PCM_OUT: u16       = 0x10; // Register box
pub const OFFSET_GLOBAL_CTRL: u16   = 0x2C; // Dword
pub const OFFSET_GLOBAL_STATUS: u16 = 0x30; // Dword

/* These are the offsets (relative to the register boxes) of the registers themselves */
pub const OFFSET_BDL_ADDR: u16      = 0x00;
pub const OFFSET_NUM_SAMPLES: u16   = 0x04;
pub const OFFSET_DESC_ENTRIES: u16  = 0x05;
pub const OFFSET_TFER_STATUS: u16   = 0x06;
pub const OFFSET_ACTUAL_TFERED: u16 = 0x08;
pub const OFFSET_NEXT_BE: u16       = 0x0A;
pub const TFER_CTRL: u16            = 0x0B;


pub const SIZE_OF_NOISE: u32        = 0x800;

pub struct AC97 {}




impl AC97 {
    pub fn new() -> Self {
        return Self {};
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        /* Find the AC97 device on PCI */
        let (bus, slot, function) = pci_device_search_by_class_subclass(0x04, 0x01)?;
        let header_type: u8 = pci_get_header_type(bus, slot, function);
    
        let mut bar0: u16 = 0x00;
        let mut bar1: u16 = 0x00;

        let control_reg: u8 = pci_read_u8(bus, slot, function, 0x04);

        /* Set to control register IO space & bus master bit which are required */
        pci_write_u8(bus, slot, function, 0x04, control_reg | 0x05);
        
        if control_reg | 0x05 != pci_read_u8(bus, slot, function, 0x04) {
            return Err("Could not set the IO & bus master bits to AC97");
        }

        match header_type {
            0x00 => {
                let header0x00 = pci_get_header_0x00(bus, slot, function)?;
                /* In this version of QEMU we know that AC97 uses Port IO */
                bar0 = pci_get_bar_address(header0x00.bar0) as u16;
                bar1 = pci_get_bar_address(header0x00.bar1) as u16;
            },
            0x01 => {
                let header0x01 = pci_get_header_0x01(bus, slot, function)?;
                /* In this version of QEMU we know that AC97 uses Port IO */
                bar0 = pci_get_bar_address(header0x01.bar0) as u16;
                bar1 = pci_get_bar_address(header0x01.bar1) as u16;
            },
            _ => {
                return Err("Wrong header type for AC97")
            }
        }

        /* CBA error handling atm, i will (maybe) add that later */

        /* Set master volume to 32 out of 64 in both channels*/
        outw(bar0 + OFFSET_VOLUME, 0x4040);
        
        /* DON'T move this code. Trust me on this one. */
        let mut rng = rand::Rng::new();
        let noise = generate_noise(rng);

        /* Set PCM output volume to +0 dB, instead of the default -inf */
        outw(bar0 + OFFSET_PCM_VOLUME, 0x8808);

        /* This code doesn't belong in an init() function but for testing purposes it will stay */
        let buf_desc: BufferDescriptor = 
            BufferDescriptor { 
                ptr: &noise as *const u16 as u32, 
                size: noise.len() as u16 / 2, // 16 bit audio = 2 bytes
                flags: 0b11 << 14 // generate interrupt after every sample; stop playing sound after this buffer is done
            };

        buf_desc.play(bar0, bar1)?;

        

        
        
        

        Ok(())
    }
}

#[repr(C)]
struct BufferDescriptor {
    ptr: u32,   // address of the sound buffer
    size: u16,  // size of the sound buffer (max 0xFFFE = 65534)
    flags: u16 // status flags
}

impl BufferDescriptor {
    fn play(&self, bar0: u16, bar1: u16) -> Result<(), &'static str> {
        let mut poll_ctr = 0;
        /* feeling cute might add error handling later */

        /* Put the address of the sound data in the buffer descriptor list */
        outd(bar1 + OFFSET_BDL_ADDR,
            (self.ptr as *const u8 as u32) 
        );

        /* Then put the number of samples: a sample is 2 bytes, 10 kB / 2 B = 5000 */
        /* Max samples is oxFFFE = 65534 */
        outw(bar1 + OFFSET_PCM_OUT + OFFSET_NUM_SAMPLES, self.size);

        /* Put the status flags in the correct register*/
        outw(bar1 + OFFSET_PCM_OUT + OFFSET_TFER_STATUS, self.flags);

        /* Set reset bit of output channel */
        outb(bar1 + 0x1B,
            inb(bar1 + 0x1B) | 0x02 
        );

        /* Poll */
        loop {
            let a = inb(bar1 + 0x1B);
            /* Check if the reset was successful */
            if a != a | 0x02 {
                break;
            } else {
                poll_ctr += 1;
            }

            if poll_ctr >= 10 {
                return Err("Took too long to set reset bit");
            }
        }
        poll_ctr = 0;

        /* Write physical position of buffer descriptor list to bar1 + 0x10 */
        let addr = as_physical_address(self as *const Self) as u32;
        if addr > 0x40000000 { // maximum u30 value
            return Err("Address too high (> 0x40000000)")
        }
        outd(bar1 + 0x10, addr);

        /* set the last valid buffer entry to 0, which is the amount of buffers - 1*/
        outb(bar1 + 0x15, 0);

        /* finally, enable data transfer */
        outb(bar1 + 0x1B,
            inb(bar1 + 0x1B) | 1
        );






        




        Ok(())
    }
}

// generate a random [u16; 1024]
// in a very inefficient way cuz cba 

fn generate_noise(mut rng: rand::Rng) -> [u16; 1024] {
    let mut i = 0;
    let mut arr = [0u16; 1024];
    while i < 1024 {
        arr[i] = rng.u32() as u16;
        i += 1;
    }

    arr
}

// wtb implementation
#[inline]
fn as_physical_address<T>(ptr: *const T) -> *const T {
    ptr
} 
