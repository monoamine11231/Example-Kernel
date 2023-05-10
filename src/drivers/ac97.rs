use crate::tooling::qemu_io::{qemu_println, qemu_print_hex};

use super::pci::{pci_device_search_by_class_subclass, pci_get_header_type, pci_get_header_0x00, pci_get_bar_address, pci_get_header_0x01};

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

        Ok(())
    }
}
