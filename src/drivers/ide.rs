use core::arch::asm;
use core::ops::IndexMut;

use heapless::String;

use super::pci::{
    pci_device_search_by_class_subclass, pci_get_bar_address, pci_get_header_0x00,
    pci_get_header_type, pci_get_progif, PCIDeviceHeader0x00,
};
use crate::tooling::qemu_io::{qemu_fmt_println, qemu_print, qemu_print_hex, qemu_println};
use crate::tooling::serial::{inb, ind, outb};

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

impl ATAStatus {
    pub fn presence(&self, state: u8) -> bool {
        return (state & (*self as u8) != 0x00) as bool;
    }
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

impl ATAError {
    pub fn presence(&self, state: u8) -> bool {
        return (state & (*self as u8) != 0x00) as bool;
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
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
    None = 0xFF,
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

impl TryFrom<u8> for ATAChannel {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(ATAChannel::Primary),
            0x01 => Ok(ATAChannel::Secondary),

            _ => Err("Wrong u8 to convert from, must be 0 or 1"),
        }
    }
}

/* Because using `as` all time is pain */
impl<T, const N: usize> core::ops::Index<ATAChannel> for [T; N] {
    type Output = T;
    #[inline]
    fn index(&self, index: ATAChannel) -> &Self::Output {
        self.index(index as usize)
    }
}

impl<T, const N: usize> core::ops::IndexMut<ATAChannel> for [T; N] {
    #[inline]
    fn index_mut(&mut self, index: ATAChannel) -> &mut Self::Output {
        self.index_mut(index as usize)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ATADrive {
    Master = 0x00,
    Slave = 0x01,
}

impl TryFrom<u8> for ATADrive {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(ATADrive::Master),
            0x01 => Ok(ATADrive::Slave),

            _ => Err("Wrong u8 to convert from, must be 0 or 1"),
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum ATADriveType {
    ATA = 0x00,
    ATAPI = 0x01,
}

impl TryFrom<u8> for ATADriveType {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(ATADriveType::ATA),
            0x01 => Ok(ATADriveType::ATAPI),

            _ => Err("Wrong u8 to convert from, must be 0 or 1"),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
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
    exists: bool,
    channel: ATAChannel,
    drive: ATADrive,
    dtype: ATADriveType,
    signature: u16,
    capabilities: u16,
    command_sets: u32,
    /* Size in sectors */
    size: u32,
    /* Model in string */
    model: String<41>,
}

impl Default for IDEDevice {
    fn default() -> Self {
        Self {
            exists: false,
            channel: ATAChannel::Primary,
            drive: ATADrive::Master,
            dtype: ATADriveType::ATA,
            signature: 0x0000,
            capabilities: 0x0000,
            command_sets: 0x00000000,
            size: 0x00000000,
            model: String::new(),
        }
    }
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
        Self {
            channels: unsafe { core::mem::zeroed() },
            devices: [
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],

            ide_buf: unsafe { core::mem::zeroed() },
            ide_irq_invoked: 1,
        }
    }
}

impl IDE {
    /* Should be replaced later */
    fn tmp_sleep(&self, channel: ATAChannel, ms: usize) {
        for i in 0..ms * 10 {
            IDE::read_chreg(self, channel, ATARegister::ControlORAltStatus);
        }
    }

    pub fn init(&mut self) {
        let mut device_count: usize = 0x00;

        self.channels[ATAChannel::Primary].no_interrupt = true;
        self.channels[ATAChannel::Secondary].no_interrupt = true;

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

            self.channels[ATAChannel::Primary].base_io = (header.bar0 & 0xFFFFFFFC) as u16;
            self.channels[ATAChannel::Primary].control_base = (header.bar1 & 0xFFFFFFFC) as u16;
            self.channels[ATAChannel::Primary].bus_master_ide =
                (header.bar4 & 0xFFFFFFFC) as u16 + 0x00;

            self.channels[ATAChannel::Secondary].base_io = (header.bar2 & 0xFFFFFFFC) as u16;
            self.channels[ATAChannel::Secondary].control_base = (header.bar3 & 0xFFFFFFFC) as u16;
            self.channels[ATAChannel::Secondary].bus_master_ide =
                (header.bar4 & 0xFFFFFFFC) as u16 + 0x08;

        /* PCI compatibility mode */
        } else {
            self.channels[ATAChannel::Primary].base_io = 0x1F0;
            self.channels[ATAChannel::Primary].control_base = 0x3F6;
            self.channels[ATAChannel::Primary].bus_master_ide = 0x00;

            self.channels[ATAChannel::Secondary].control_base = 0x376;
            self.channels[ATAChannel::Secondary].base_io = 0x170;
            self.channels[ATAChannel::Secondary].bus_master_ide = 0x08;
        }

        /* Disable IRQ */
        IDE::write_chreg(
            &self,
            ATAChannel::Primary,
            ATARegister::ControlORAltStatus,
            2,
        );
        IDE::write_chreg(
            &self,
            ATAChannel::Secondary,
            ATARegister::ControlORAltStatus,
            2,
        );

        /* Detect ATA device */
        for i in 0u8..2u8 {
            for j in 0u8..2u8 {
                let mut err: u8 = 0;
                let channel: ATAChannel = ATAChannel::try_from(i).unwrap();
                let drive: ATADrive = ATADrive::try_from(j).unwrap();

                /* Mark firstly that the drive does not exist */
                self.devices[device_count].exists = false;

                /* Select drive */
                IDE::write_chreg(&self, channel, ATARegister::HDDEvsel, 0x0A | (j << 4));

                /* Sleep 1ms by reading alt status port 10 times */
                IDE::tmp_sleep(&self, channel, 1);

                /* Send ATA identify command */
                IDE::write_chreg(
                    &self,
                    channel,
                    ATARegister::CommandORStatus,
                    ATACommand::Identity as u8,
                );

                /* Sleep for 1ms */
                IDE::tmp_sleep(&self, channel, 1);

                /* If status == 0: no device */
                if IDE::read_chreg(&self, channel, ATARegister::CommandORStatus) == 0x00 {
                    continue;
                }

                loop {
                    let state = IDE::read_chreg(&self, channel, ATARegister::CommandORStatus);
                    /* Not ATA */
                    if ATAStatus::Error.presence(state) {
                        err = 0x01;
                        break;
                    }

                    if !ATAStatus::Busy.presence(state)
                        && ATAStatus::DataRequestReady.presence(state)
                    {
                        err = 0x00;
                        break;
                    }
                }

                /* If not ATA: move to the next device. We do not need ATAPI */
                if err != 0x00 {
                    continue;
                }

                /* We read the identification space for the device */
                IDE::read_to_buffer(self, channel, ATARegister::Data, 128);

                let ide_buf: *const u8 = unsafe { self.ide_buf.as_ptr() as *const u8 };

                self.devices[device_count].exists = true;
                self.devices[device_count].dtype = ATADriveType::ATA;
                self.devices[device_count].channel = channel;
                self.devices[device_count].drive = drive;
                self.devices[device_count].signature = unsafe {
                    *(ide_buf.offset(ATAIdentitySpace::DeviceType as isize) as *const u16)
                };
                self.devices[device_count].capabilities = unsafe {
                    *(ide_buf.offset(ATAIdentitySpace::Capabilities as isize) as *const u16)
                };
                self.devices[device_count].command_sets = unsafe {
                    *(ide_buf.offset(ATAIdentitySpace::CommandSets as isize) as *const u32)
                };

                if (self.devices[device_count].command_sets & (1 << 26)) != 0x00 {
                    /* Device uses 48-bit LBA addressing */
                    self.devices[device_count].size = unsafe {
                        *(ide_buf.offset(ATAIdentitySpace::MaxLBAExt as isize) as *const u32)
                    }
                } else {
                    /* CHS or 28-bit addressing */
                    self.devices[device_count].size = unsafe {
                        *(ide_buf.offset(ATAIdentitySpace::MaxLBA as isize) as *const u32)
                    }
                }

                /* Copy model of device */
                for k in (0usize..40usize).step_by(2) {
                    self.devices[device_count]
                        .model
                        .push(self.ide_buf[ATAIdentitySpace::Model as usize + k + 1] as char);
                    self.devices[device_count]
                        .model
                        .push(self.ide_buf[ATAIdentitySpace::Model as usize + k] as char);
                }

                device_count += 1;
            }
        }

        // for i in 0usize..4usize {
        //     if self.devices[i].exists {
        //         qemu_println("Found device:");
        //         qemu_println(self.devices[i].model.as_str());
        //         qemu_print_hex(self.devices[i].size);
        //     }
        // }
    }

    pub fn read_chreg(&self, channel: ATAChannel, register: ATARegister) -> u8 {
        let ru8: u8 = register as u8;

        if ru8 > 0x07 && ru8 < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                0x80 | self.channels[channel].no_interrupt as u8,
            );
        }

        let result = match ru8 {
            x if ru8 < 0x08 => (inb(self.channels[channel].base_io + (register as u16) - 0x00)),
            x if ru8 < 0x0C => (inb(self.channels[channel].base_io + (register as u16) - 0x06)),
            x if ru8 < 0x0E => {
                (inb(self.channels[channel].control_base + (register as u16) - 0x0A))
            }
            x if ru8 < 0x16 => {
                (inb(self.channels[channel].bus_master_ide + (register as u16) - 0x0E))
            }
            _ => 0x00,
        };

        // if (register as u8) < 0x08 {
        //     result = inb(self.channels[channel].base_io + (register as u16) - 0x00);
        // } else if (register as u8) < 0x0C {
        //     result = inb(self.channels[channel].base_io + (register as u16) - 0x06);
        // } else if (register as u8) < 0x0E {
        //     result = inb(self.channels[channel].control_base + (register as u16) - 0x0A);
        // } else if (register as u8) < 0x16 {
        //     result = inb(self.channels[channel].bus_master_ide + (register as u16) - 0x0E);
        // }

        if ru8 > 0x07 && ru8 < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                self.channels[channel].no_interrupt as u8,
            );
        }

        return result;
    }

    pub fn write_chreg(&self, channel: ATAChannel, register: ATARegister, data: u8) {
        let ru8: u8 = register as u8;
        if ru8 > 0x07 && ru8 < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                0x80 | self.channels[channel].no_interrupt as u8,
            )
        }

        match ru8 {
            x if ru8 < 0x08 => outb(
                self.channels[channel].base_io + (register as u16) - 0x00,
                data,
            ),
            x if ru8 < 0x0C => outb(
                self.channels[channel].base_io + (register as u16) - 0x06,
                data,
            ),
            x if ru8 < 0x0E => outb(
                self.channels[channel].control_base + (register as u16) - 0x0A,
                data,
            ),
            x if ru8 < 0x16 => {
                (outb(
                    self.channels[channel].bus_master_ide + (register as u16) - 0x0E,
                    data,
                ))
            }
            _ => (),
        };

        // if ru8 < 0x08 {
        //     outb(
        //         self.channels[channel].base_io + (register as u16) - 0x00,
        //         data,
        //     );
        // } else if ru8 < 0x0C {
        //     outb(
        //         self.channels[channel].base_io + (register as u16) - 0x06,
        //         data,
        //     );
        // } else if ru8 < 0x0E {
        //     outb(
        //         self.channels[channel].control_base + (register as u16) - 0x0A,
        //         data,
        //     );
        // } else if (register as u8) < 0x16 {
        //     outb(
        //         self.channels[channel].bus_master_ide + (register as u16) - 0x0E,
        //         data,
        //     );
        // }

        if ru8 > 0x07 && ru8 < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                self.channels[channel].no_interrupt as u8,
            )
        }
    }

    pub fn read_to_buffer(&mut self, channel: ATAChannel, register: ATARegister, dwords: u32) {
        let ru8: u8 = register as u8;
        if ru8 > 0x07 && ru8 < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                0x80 | self.channels[channel].no_interrupt as u8,
            );
        }

        unsafe {
            let mut bufd: *mut u32 = unsafe { (self.ide_buf.as_mut_ptr() as *mut u32) };

            match ru8 {
                x if ru8 < 0x08 => {
                    for offset in 0..dwords {
                        let res: u32 =
                            ind(self.channels[channel].base_io + (register as u16) - 0x00);
                        *bufd.offset(offset as isize) = res;
                    }
                }
                x if ru8 < 0x0C => {
                    for offset in 0..dwords {
                        let mut res: u32 =
                            ind(self.channels[channel].base_io + (register as u16) - 0x06);
                        *bufd.offset(offset as isize) = res
                    }
                }
                x if ru8 < 0x0E => {
                    for offset in 0..dwords {
                        let mut res: u32 =
                            ind(self.channels[channel].base_io + (register as u16) - 0x0A);

                        *bufd.offset(offset as isize) = res
                    }
                }
                x if ru8 < 0x16 => {
                    for offset in 0..dwords {
                        let mut res: u32 =
                            ind(self.channels[channel].base_io + (register as u16) - 0x0E);

                        *bufd.offset(offset as isize) = res
                    }
                }
                _ => (),
            };
        };

        // if (register as u8) < 0x08 {
        //     unsafe {
        //         for offset in 0..dwords {
        //             let res: u32 = ind(self.channels[channel].base_io + (register as u16) - 0x00);
        //             *bufd.offset(offset as isize) = res;
        //         }
        //     };
        // } else if (register as u8) < 0x0C {
        //     unsafe {
        //         for offset in 0..dwords {
        //             let mut res: u32 =
        //                 ind(self.channels[channel].base_io + (register as u16) - 0x06);
        //             *bufd.offset(offset as isize) = res
        //         }
        //     };
        // } else if (register as u8) < 0x0E {
        //     unsafe {
        //         for offset in 0..dwords {
        //             let mut res: u32 =
        //                 ind(self.channels[channel].base_io + (register as u16) - 0x0A);

        //             *bufd.offset(offset as isize) = res
        //         }
        //     };
        // } else if (register as u8) < 0x16 {
        //     unsafe {
        //         for offset in 0..dwords {
        //             let mut res: u32 =
        //                 ind(self.channels[channel].base_io + (register as u16) - 0x0E);

        //             *bufd.offset(offset as isize) = res
        //         }
        //     };
        // }

        if ru8 > 0x07 && ru8 < 0x0C {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::ControlORAltStatus,
                self.channels[channel].no_interrupt as u8,
            );
        }
    }

    pub fn polling(&self, channel: ATAChannel) -> Result<u8, &'static str> {
        /* Delay 400ns by reading alt status port 4 times, which takes in total 400ns */
        for i in 0..4 {
            IDE::read_chreg(self, channel, ATARegister::ControlORAltStatus);
        }

        /* Wait until not busy */
        while ATAStatus::Busy.presence(IDE::read_chreg(
            &self,
            channel,
            ATARegister::CommandORStatus,
        )) {}

        let state: u8 = IDE::read_chreg(&self, channel, ATARegister::CommandORStatus);
        if ATAStatus::Error.presence(state) {
            /* General error */
            return Err("General error happened when polling the drive!");
        }

        if ATAStatus::DriveWriteFault.presence(state) {
            /* Drive write fault */
            return Err("Drive write fault happened when polling the drive!");
        }

        if !ATAStatus::DataRequestReady.presence(state) {
            /* Data request ready bit should be set */
            return Err("The data request ready bit was not set when polling the drive!");
        }

        Ok(0)
    }

    pub fn print_error(&self, drive: usize, err: u8) {
        qemu_print("IDE ERROR: [");
        qemu_print(["Primary", "Secondary"][self.devices[drive].channel as usize]);
        qemu_print(" | ");
        qemu_print(["Master", "Slave"][self.devices[drive].drive as usize]);
        qemu_print(" | ");
        qemu_print(self.devices[drive].model.as_str());
        qemu_print(": ");

        if err == 1 {
            qemu_println("Device fault");
        } else if err == 2 {
            let state: u8 = IDE::read_chreg(
                &self,
                self.devices[drive].channel,
                ATARegister::ErrorORFeatures,
            );
            match state {
                x if ATAError::NoAddressMark.presence(state) => {
                    (qemu_println("No address mark found"))
                }
                x if ATAError::TrackZeroNotFound.presence(state) => {
                    (qemu_println("No media or media error"))
                }
                x if ATAError::CommandAborted.presence(state) => (qemu_println("Command aborted")),
                x if ATAError::MediaChangeRequest.presence(state) => {
                    (qemu_println("No media or media error"))
                }
                x if ATAError::IDMarkNotFound.presence(state) => {
                    (qemu_println("ID mark not found"))
                }
                x if ATAError::MediaChanged.presence(state) => {
                    (qemu_println("No media or media error"))
                }
                x if ATAError::UncorrectableData.presence(state) => {
                    (qemu_println("Uncorrectable data error"))
                }
                x if ATAError::BadBlock.presence(state) => (qemu_println("Bad sectors")),
                _ => (),
            }
        } else if err == 3 {
            qemu_println("Reads nothing");
        } else if err == 4 {
            qemu_println("Write protected");
        }
    }

    /// `direction`: Read, Write; `drive`: Drive #; `address`: LBA48, LBA28 or CHS; `nsects`: <256;
    /// `edi`: memory address
    pub fn ata_access_pio(
        &mut self,
        direction: ATADirection,
        drive: u8,
        address: u64,
        nsects: u8,
        edi: u32,
    ) {
        let channel: ATAChannel = self.devices[drive as usize].channel;
        let is_slave: u8 = self.devices[drive as usize].drive as u8;
        /* 0: CHS; 1: LBA28; 2: LBA48 */
        let mut addressing_mode: u8 = 0x00;

        let mut head = 0x00;
        let mut address_sliced: [u8; 6] = [0u8; 6];

        IDE::write_chreg(&self, channel, ATARegister::ControlORAltStatus, 0x02);
        self.ide_irq_invoked = 0x00;
        self.channels[channel].no_interrupt = false;

        /* LBA48 */
        if address >= 0x10000000 {
            addressing_mode = 0x02;

            address_sliced[0] = ((address & 0x00000000000000FF) >> 0) as u8;
            address_sliced[1] = ((address & 0x000000000000FF00) >> 8) as u8;
            address_sliced[2] = ((address & 0x0000000000FF0000) >> 16) as u8;
            address_sliced[3] = ((address & 0x00000000FF000000) >> 24) as u8;
            address_sliced[4] = ((address & 0x000000FF00000000) >> 32) as u8;
            address_sliced[5] = ((address & 0x0000FF0000000000) >> 40) as u8;

            head = 0x00;
        /* LBA28 */
        } else if (self.devices[drive as usize].capabilities & 0x200) != 0x00 {
            addressing_mode = 0x01;

            address_sliced[0] = ((address & 0x00000000000000FF) >> 0) as u8;
            address_sliced[1] = ((address & 0x000000000000FF00) >> 8) as u8;
            address_sliced[2] = ((address & 0x0000000000FF0000) >> 16) as u8;
            address_sliced[3] = 0x00u8;
            address_sliced[4] = 0x00u8;
            address_sliced[5] = 0x00u8;

            head = ((address & 0xF000000) >> 24) as u8;
        /* CHS */
        } else {
            addressing_mode = 0x00;

            let sector: u8 = ((address % 63) + 1) as u8;
            let cylinder: u16 = ((address + 1 - sector as u64) / (16 * 63)) as u16;

            address_sliced[0] = sector;
            address_sliced[1] = ((cylinder >> 0) & 0xFF) as u8;
            address_sliced[2] = ((cylinder >> 8) & 0xFF) as u8;
            address_sliced[3] = 0;
            address_sliced[4] = 0;
            address_sliced[5] = 0;

            head = ((address + 1 - sector as u64) % (16 * 63) / 63) as u8;
        }

        /* Wait until not busy */
        while ATAStatus::Busy.presence(IDE::read_chreg(
            &self,
            channel,
            ATARegister::CommandORStatus,
        )) {}

        if addressing_mode == 0x00 {
            /* We indicate CHS mode */
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::HDDEvsel,
                0xA0 | (is_slave << 4) | head,
            );
        /* We indicate LBA mode */
        } else {
            IDE::write_chreg(
                &self,
                channel,
                ATARegister::HDDEvsel,
                0xE0 | (is_slave << 4) | head,
            );
        }

        /* We write the address to the registers */
        /* If LBA48 is used, write the upper LBA bytes */
        if addressing_mode == 0x02 {
            IDE::write_chreg(&self, channel, ATARegister::Seccount1, 0x00);
            IDE::write_chreg(&self, channel, ATARegister::LBA3, address_sliced[3]);
            IDE::write_chreg(&self, channel, ATARegister::LBA4, address_sliced[4]);
            IDE::write_chreg(&self, channel, ATARegister::LBA5, address_sliced[5]);
        }
        /* Write the remaining ones */
        IDE::write_chreg(&self, channel, ATARegister::Seccount0, nsects);
        IDE::write_chreg(&self, channel, ATARegister::LBA0, address_sliced[0]);
        IDE::write_chreg(&self, channel, ATARegister::LBA1, address_sliced[1]);
        IDE::write_chreg(&self, channel, ATARegister::LBA2, address_sliced[2]);

        let command = match addressing_mode {
            /* For CHS or LBA28 mode */
            0 | 1 if direction == ATADirection::Read => ATACommand::ReadPIO,
            0 | 1 if direction == ATADirection::Write => ATACommand::WritePIO,

            /* LBA 48 */
            2 if direction == ATADirection::Read => ATACommand::ReadPIOExt,
            2 if direction == ATADirection::Write => ATACommand::WritePIOExt,

            _ => ATACommand::None,
        };

        if command == ATACommand::None {
            panic!("Wrong command given to IDE ATA access function!");
        }

        IDE::write_chreg(&self, channel, ATARegister::CommandORStatus, command as u8);

        let mut edi_offset: u32 = edi;
        if direction == ATADirection::Read {
            for i in 0..nsects {
                IDE::polling(&self, channel).unwrap();
                unsafe {
                    asm!(
                        "rep insd",
                        in("rcx") 128,
                        in("dx") self.channels[channel].base_io,
                        in("rdi") edi_offset,
                        options(nostack, preserves_flags, nomem)
                    );
                    /* One sector read */
                    edi_offset += 512;
                };
            }
            return;
        }

        /* Write PIO */
        for i in 0..nsects {
            IDE::polling(&self, channel);
            unsafe {
                asm!(
                    "rep outsd",
                    in("rcx") 128,
                    in("dx") self.channels[channel].base_io,
                    in("rsi") edi_offset,
                    options(nostack, preserves_flags, nomem)
                )
            };
        }
    }
}
