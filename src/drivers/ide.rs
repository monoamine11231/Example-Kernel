use super::pci::{
    pci_device_search_by_class_subclass, pci_get_bar_address, pci_get_header_0x00,
    pci_get_header_type, pci_get_progif, PCIDeviceHeader0x00,
};
use crate::tooling::qemu_io::{qemu_print_hex, qemu_println};
use crate::tooling::serial::{inb, outb, ind};

pub const IDE_ATA: u8 = 0x00;
pub const IDE_ATAPI: u8 = 0x01;

pub const ATA_MASTER: u8 = 0x00;
pub const ATA_SLAVE: u8 = 0x01;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ATAStatus {
    Busy = 0x80,
    DriveReady = 0x40,
    DriveWriteFault = 0x20,
    DriveSeekComplete = 0x10,
    DataRequestReady = 0x08,
    CorrectedData = 0x04,
    Index = 0x02,
    Error = 0x01,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ATAError {
    BadBlock = 0x80,
    UncorrectableData = 0x40,
    MediaChanged = 0x20,
    IDMarkNotFound = 0x10,
    MediaChangeRequest = 0x08,
    CommandAborted = 0x04,
    TrackZeroNotFound = 0x02,
    NoAddressMark = 0x01,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ATACommand {
    ReadPIO = 0x20,
    ReadPIOExt = 0x24,
    ReadDMA = 0xC8,
    ReadDMAExt = 0x25,
    WritePIO = 0x30,
    WritePIOExt = 0x34,
    WriteDMA = 0xCA,
    WriteDMAExt = 0x35,
    CacheFlush = 0xE7,
    CacheFlushExt = 0xEA,
    Packet = 0xA0,
    IdentityPacket = 0xA1,
    Identity = 0xEC,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ATAPICommand {
    CommandRead = 0xA8,
    CommandEject = 0x1B,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ATAIdentitySpace {
    DeviceType = 0x00,
    Cylinders = 0x02,
    Heads = 0x06,
    Sectors = 0x0C,
    Serial = 0x14,
    Model = 0x36,
    Capabilities = 0x62,
    FieldValid = 0x6A,
    MaxLBA = 0x78,
    CommandSets = 0xA4,
    MaxLBAExt = 0xC8,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ATARegister {
    Data = 0x00,
    ErrorORFeatures = 0x01,
    Seccount0 = 0x02,
    LBA0 = 0x03,
    LBA1 = 0x04,
    LBA2 = 0x05,
    HDDEvsel = 0x06,
    CommandORStatus = 0x07,
    Seccount1 = 0x08,
    LBA3 = 0x09,
    LBA4 = 0x0A,
    LBA5 = 0x0B,
    ControlORAltStatus = 0x0C,
    DEVAddress = 0x0D,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ATAChannel {
    Primary = 0x00,
    Secondary = 0x01,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ATADirection {
    Read = 0x00,
    Write = 0x01,
}

pub struct IDEChannel {
    base_io: u16,
    control_base: u16,

    bus_master_ide: u16,
    no_interrupt: bool,
}

pub struct IDEDevice {
    exists: u8,
    /* 0: Primary; 1: Secondary */
    channel: u8,
    /* 0: Master; 1: Slave */
    drive: u8,
    /* 0: ATA; 1: ATAPI */
    dtype: u16,
    signature: u16,
    capabilities: u16,
    command_sets: u32,
    /* Size in sectors */
    size: u32,
    /* Model in string */
    model: [u8; 41],
}
pub struct IDE {
    channels: [IDEChannel; 2],
    devices: [IDEDevice; 4],

    ide_buf: [u8; 2048],
    ide_irq_invoked: u8,
}

/* Rust god forgive me for this sin */
impl Default for IDE {
    fn default() -> Self {
        Self{
            channels: unsafe { core::mem::zeroed() },
            devices: unsafe { core::mem::zeroed() },

            ide_buf: unsafe { core::mem::zeroed() },
            ide_irq_invoked: 0
        }
    }
}

impl IDE {
    pub fn init(&mut self) {
        let ide_dev: (u8, u8, u8) = pci_device_search_by_class_subclass(0x01, 0x01).unwrap();

        let ide_bus: u8 = ide_dev.0;
        let ide_slot: u8 = ide_dev.1;
        let ide_function: u8 = ide_dev.2;

        let ide_progif: u8 = pci_get_progif(ide_bus, ide_slot, ide_function);

        /* PCI native mode */
        if ide_progif & 0x01 == 1 {
            let header_type: u8 = pci_get_header_type(ide_bus, ide_slot, ide_function);
            if header_type != 0x00 {
                panic!("PCI IDE device found and used with a header type other than 0x00");
            }

            let header: PCIDeviceHeader0x00 =
                pci_get_header_0x00(ide_bus, ide_slot, ide_function).unwrap();

            self.channels[ATAChannel::Primary as usize].base_io = (header.bar0 & 0xFFFFFFFC) as u16;
            self.channels[ATAChannel::Primary as usize].control_base = (header.bar1 & 0xFFFFFFFC) as u16;
            self.channels[ATAChannel::Primary as usize].bus_master_ide =
                (header.bar4 & 0xFFFFFFFC) as u16 + 0x00;

            self.channels[ATAChannel::Secondary as usize].base_io = (header.bar2 & 0xFFFFFFFC) as u16;
            self.channels[ATAChannel::Secondary as usize].control_base =
                (header.bar3 & 0xFFFFFFFC) as u16;
            self.channels[ATAChannel::Secondary as usize].bus_master_ide =
                (header.bar4 & 0xFFFFFFFC) as u16 + 0x08;

        /* PCI compatibility mode */
        } else {
            self.channels[ATAChannel::Primary as usize].base_io = 0x1F0;
            self.channels[ATAChannel::Primary as usize].control_base = 0x3F6;
            self.channels[ATAChannel::Primary as usize].bus_master_ide = 0x00;

            self.channels[ATAChannel::Secondary as usize].control_base = 0x376;
            self.channels[ATAChannel::Secondary as usize].base_io = 0x170;
            self.channels[ATAChannel::Secondary as usize].bus_master_ide = 0x08;
        }
    }

    pub fn read_chreg(&self, channel: ATAChannel, register: ATARegister) -> u8 {
        let mut result: u8 = 0x00;
        if (register as u8) > 0x07 && (register as u8) < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                0x80 | self.channels[channel as usize].no_interrupt as u8,
            );
        }

        if (register as u8) < 0x08 {
            result = inb(self.channels[channel as usize].base_io + (register as u16) - 0x00);
        } else if (register as u8) < 0x0C {
            result = inb(self.channels[channel as usize].base_io + (register as u16) - 0x06);
        } else if (register as u8) < 0x0E {
            result = inb(self.channels[channel as usize].control_base + (register as u16) - 0x0A);
        } else if (register as u8) < 0x16 {
            result = inb(self.channels[channel as usize].bus_master_ide + (register as u16) - 0x0E);
        }

        if (register as u8) > 0x07 && (register as u8) < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                self.channels[channel as usize].no_interrupt as u8,
            );
        }

        return result;
    }

    pub fn write_chreg(&self, channel: ATAChannel, register: ATARegister, data: u8) {
        if (register as u8) > 0x07 && (register as u8) < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                0x80 | self.channels[channel as usize].no_interrupt as u8,
            )
        }

        if (register as u8) < 0x08 {
            outb(
                self.channels[channel as usize].base_io + (register as u16) - 0x00,
                data,
            );
        } else if (register as u8) < 0x0C {
            outb(
                self.channels[channel as usize].base_io + (register as u16) - 0x06,
                data,
            );
        } else if (register as u8) < 0x0E {
            outb(
                self.channels[channel as usize].control_base + (register as u16) - 0x0A,
                data,
            );
        } else if (register as u8) < 0x16 {
            outb(
                self.channels[channel as usize].bus_master_ide + (register as u16) - 0x0E,
                data,
            );
        }

        if (register as u8) > 0x07 && (register as u8) < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                self.channels[channel as usize].no_interrupt as u8,
            )
        }
    }

    pub fn read_to_buffer(&self, channel: ATAChannel, register: ATARegister, buffer: u64, quads: u32) {
        if (register as u8) > 0x07 && (register as u8) < 0x0C {
            
        }
    }

}
