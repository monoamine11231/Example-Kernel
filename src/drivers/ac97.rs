use crate::tooling::qemu_io::{qemu_println, qemu_print_hex};
use crate::tooling::serial::*;
use crate::qemu_print;
use super::pci::{pci_device_search_by_class_subclass, pci_get_header_type, pci_get_header_0x00, pci_get_bar_address, pci_get_header_0x01, pci_read_u8, pci_write_u8};
use crate::misc::rand;
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

        /* Put the address of the sound data in the buffer descriptor list */
        outd(bar1 + OFFSET_BDL_ADDR,
            (&noise as *const u8 as u32) 
        );

        /* Then put the number of samples: a sample is 2 bytes, 10 kB / 2 B = 5000 */
        outw(bar1 + OFFSET_NUM_SAMPLES, 10000 / 2);

        /* Still a lot more to do here, lol */

        

        Ok(())
    }
}

// generate a random [u8; 10000]
// in a very inefficient way cuz cba 

fn generate_noise(mut rng: rand::Rng) -> [u8; 10000] {
    let mut i = 0;
    let mut arr = [0u8; 10000];
    while i < 10000 {
        arr[i] = rng.u32() as u8;
        i += 1;
    }

    arr
}
