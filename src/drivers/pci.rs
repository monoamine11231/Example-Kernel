use crate::tooling::{
    qemu_io::qemu_print_hex,
    serial::{ind, outb, outd},
};

/* Header structs for PCI devices */

/// PCI Device Header that exists on all devices prior to their type specific header
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct PCIDeviceCommonHeader {
    vendor_id: u16,
    device_id: u16,

    command: u16,
    status: u16,

    revision_id: u8,
    prog_if: u8,
    subclass: u8,
    class_code: u8,

    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8,
}

/// PCI Device Header for devices with header type of 0x00.
/// This header is followed after the common PCI device header `PCIDeviceCommonHeader`
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct PCIDeviceHeader0x00 {
    pub bar0: u32,
    pub bar1: u32,
    pub bar2: u32,
    pub bar3: u32,
    pub bar4: u32,
    pub bar5: u32,

    pub cardbus_cis_pointer: u32,

    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,

    pub expansion_rom_bar: u32,

    pub capabilities_pointer: u8,
    reserved0: u8,
    reserved1: u16,

    reserved2: u32,

    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8,
}

/// PCI Device Header for devices with header type of 0x01.
/// This header is followed after the common PCI device header `PCIDeviceCommonHeader`
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct PCIDeviceHeader0x01 {
    pub bar0: u32,
    pub bar1: u32,

    pub primary_bus_number: u8,
    pub secondary_bus_number: u8,
    pub subordinate_bus_number: u8,
    pub secondary_latency_timer: u8,

    pub io_base: u8,
    pub io_limit: u8,
    pub secondary_status: u16,

    pub memory_base: u16,
    pub memory_limit: u16,

    pub prefetchable_memory_base: u16,
    pub prefetchable_memory_limit: u16,

    pub prefetchable_base_upper_32: u32,
    pub prefetchable_limit_upper_32: u32,

    pub io_base_upper_16: u16,
    pub io_limit_upper_16: u16,

    pub capability_pointer: u8,
    reserved0: u8,
    reserved1: u16,

    pub expansion_rom_bar: u32,

    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bridge_control: u16,
}

/// PCI Device Header for devices with header type of 0x02.
/// This header is followed after the common PCI device header `PCIDeviceCommonHeader`
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct PCIDeviceHeader0x02 {
    cardbus_socket_bar: u32,

    offset_capability_list: u8,
    reserved0: u8,
    secondary_status: u16,

    pci_bus_number: u8,
    cardbus_bus_number: u8,
    subordinate_bus_number: u8,
    cardbus_latency_timer: u8,

    mem_bar0: u32,
    mem_limit0: u32,

    mem_bar1: u32,
    mem_limit1: u32,

    io_bar0: u32,
    io_limit0: u32,

    io_bar1: u32,
    io_limit1: u32,

    interrupt_line: u8,
    interrupt_pin: u8,
    bridge_control: u16,

    subsystem_device_id: u16,
    subsystem_vendor_id: u16,

    pc_card_legacy_mode_bar: u32,
}

/// Read a DWORD at given bus #, slot #, function # and with a given offset in bytes
/// which is masked by 0xFC (padded by 0x04)
pub fn pci_read_u32(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
    let base_addr: u32 = 1 << 31; /* Enable bit (bit 31) */

    let mut final_addr: u32 = base_addr;

    final_addr |= (bus as u32) << 16; /* Bus number (bits 23-16) */
    final_addr |= ((slot as u32) & 0x1f) << 11; /* Device number (bits 15-11) */
    final_addr |= ((function as u32) & 0x03) << 8; /* Function number (bits 10-8) */
    final_addr |= (offset as u32) & 0xFC; /* Register offset (bits 7-0) */

    outd(0xCF8, final_addr);

    return ind(0xCFC);
}

/// Read a WORD at given bus #, slot #, function # and with a given offset in bytes
/// which is masked by 0x02 (padded by 0x02)
pub fn pci_read_u16(bus: u8, slot: u8, function: u8, offset: u8) -> u16 {
    let in_raw: u32 = pci_read_u32(bus, slot, function, offset);

    /* Extracts the word requested (offset within the register) */
    let in_16: u16 = (in_raw >> ((offset & 0x02) * 8) & 0xFFFF) as u16;
    return in_16;
}

/// Read a BYTE at given bus #, slot #, function # and with a given offset in bytes
pub fn pci_read_u8(bus: u8, slot: u8, function: u8, offset: u8) -> u8 {
    let in_raw: u16 = pci_read_u16(bus, slot, function, offset);
    return (in_raw >> ((offset & 0x01) * 8) & 0xFF) as u8;
}

pub fn pci_write_u8(bus: u8, slot: u8, function: u8, offset: u8, data: u8) {
    let base_addr: u32 = 1 << 31; /* Enable bit (bit 31) */

    let mut final_addr: u32 = base_addr;

    final_addr |= (bus as u32) << 16; /* Bus number (bits 23-16) */
    final_addr |= ((slot as u32) & 0x1f) << 11; /* Device number (bits 15-11) */
    final_addr |= ((function as u32) & 0x03) << 8; /* Function number (bits 10-8) */
    final_addr |= (offset as u32) & 0xFF; /* Register offset (bits 7-0) */

    outd(0xCF8, final_addr);

    /* Write to offset */
    outb(0xCFC, data);
}

/* Some important fields extracting functions */

/// Extracts and returns the Vendor ID at given bus #, slot #, function #
pub fn pci_get_vendor_id(bus: u8, slot: u8, function: u8) -> u16 {
    return pci_read_u16(bus, slot, function, 0x00);
}

/// Extracts and returns the Device ID at given bus #, slot #, function #
pub fn pci_get_device_id(bus: u8, slot: u8, function: u8) -> u16 {
    return pci_read_u16(bus, slot, function, 0x02);
}

/// Extracts and returns the class code at given bus #, slot #, function #
pub fn pci_get_class(bus: u8, slot: u8, function: u8) -> u8 {
    return pci_read_u8(bus, slot, function, 0xB);
}

pub fn pci_get_subclass(bus: u8, slot: u8, function: u8) -> u8 {
    return pci_read_u8(bus, slot, function, 0xA);
}

/// Extracts and returns the prog IF byte at given bus #, slot #, function #
pub fn pci_get_progif(bus: u8, slot: u8, function: u8) -> u8 {
    return pci_read_u8(bus, slot, function, 0x9);
}

/// Extracts and returns the header type at given bus #, slot #, function #
pub fn pci_get_header_type(bus: u8, slot: u8, function: u8) -> u8 {
    return pci_read_u8(bus, slot, function, 0xE);
}

/// Extracts and returns the common PCI device header as a struct,
/// at given bus #, slot #, function #
pub fn pci_get_common_header(bus: u8, slot: u8, function: u8) -> PCIDeviceCommonHeader {
    let mut header_buffer = [0 as u32; 4];
    header_buffer[0] = pci_read_u32(bus, slot, function, 0x00);
    header_buffer[1] = pci_read_u32(bus, slot, function, 0x04);
    header_buffer[2] = pci_read_u32(bus, slot, function, 0x08);
    header_buffer[3] = pci_read_u32(bus, slot, function, 0x0C);

    let header: PCIDeviceCommonHeader = unsafe { *(header_buffer.as_ptr() as *const _) };
    return header;
}

/// Extracts and returns the PCI device header of type 0x00 as a struct,
/// at given bus #, slot #, function #. Tests if the header type of the device is valid
/// and if the device exists at all
pub fn pci_get_header_0x00(
    bus: u8,
    slot: u8,
    function: u8,
) -> Result<PCIDeviceHeader0x00, &'static str> {
    if pci_get_vendor_id(bus, slot, function) == 0xFFFF {
        /*Device with unvalid vendor ID was given*/
        return Err("Device with unvalid vendor ID was given");
    }

    if pci_get_header_type(bus, slot, function) != 0x00 {
        /*Device with header type of 0x00 was expected*/
        return Err("Device with header type of 0x00 was expected");
    }

    let mut header_buffer: [u32; 12] = [0 as u32; 12];
    let mut offset: u8 = 0x10;
    let mut i: usize = 0x00;

    /* Read the header */
    while offset <= 0x3C {
        header_buffer[i] = pci_read_u32(bus, slot, function, offset);

        i += 1;
        offset += 0x04;
    }

    let header: PCIDeviceHeader0x00 = unsafe { *(header_buffer.as_ptr() as *const _) };
    Ok(header)
}

/// Extracts and returns the PCI device header of type 0x01 as a struct,
/// at given bus #, slot #, function #. Tests if the header type of the device is valid
/// and if the device exists at all
pub fn pci_get_header_0x01(
    bus: u8,
    slot: u8,
    function: u8,
) -> Result<PCIDeviceHeader0x01, &'static str> {
    if pci_get_vendor_id(bus, slot, function) == 0xFFFF {
        /*Device with unvalid vendor ID was given*/
        return Err("Device with unvalid vendor ID was given");
    }

    if pci_get_header_type(bus, slot, function) != 0x01 {
        /*Device with header type of 0x01 was expected*/
        return Err("Device with header type of 0x01 was expected");
    }

    let mut header_buffer: [u32; 12] = [0 as u32; 12];
    let mut offset: u8 = 0x10;
    let mut i: usize = 0x00;

    /* Read the header */
    while offset <= 0x3C {
        header_buffer[i] = pci_read_u32(bus, slot, function, offset);

        i += 1;
        offset += 0x04;
    }

    let header: PCIDeviceHeader0x01 = unsafe { *(header_buffer.as_ptr() as *const _) };
    Ok(header)
}

/// Extracts and returns the PCI device header of type 0x02 as a struct,
/// at given bus #, slot #, function #. Tests if the header type of the device is valid
/// and if the device exists at all
pub fn pci_get_header_0x02(
    bus: u8,
    slot: u8,
    function: u8,
) -> Result<PCIDeviceHeader0x02, &'static str> {
    if pci_get_vendor_id(bus, slot, function) == 0xFFFF {
        /*Device with unvalid vendor ID was given*/
        return Err("Device with unvalid vendor ID was given");
    }

    if pci_get_header_type(bus, slot, function) != 0x02 {
        /*Device with header type of 0x02 was expected*/
        return Err("Device with header type of 0x02 was expected");
    }

    let mut header_buffer: [u32; 14] = [0 as u32; 14];
    let mut offset: u8 = 0x10;
    let mut i: usize = 0x00;

    /* Read the header */
    while offset <= 0x44 {
        header_buffer[i] = pci_read_u32(bus, slot, function, offset);

        i += 1;
        offset += 0x04;
    }

    let header: PCIDeviceHeader0x02 = unsafe { *(header_buffer.as_ptr() as *const _) };
    Ok(header)
}

/// Returns the actual address determined in BAR. Accepts both mem and IO BAR
pub fn pci_get_bar_address(bar: u32) -> u32 {
    /* IO BAR is last bit is set */
    if bar & 0x01 == 0x01 {
        /* Ignore the last 2 bits */
        return bar & 0xFFFFFFFC;
    }

    /* Ignore the last 4 bits if mem BAR */
    return bar & 0xFFFFFFF0;
}

/// Returns the bus #, slot # and function # in a triple when found a device with the
/// given class code. If such device was not found, returns (0xFF, 0xFF, 0xFF) as a
/// signifier. This function brute forces through all bus lanes and slots.
pub fn pci_device_search_by_class_subclass(
    class: u8,
    subclass: u8,
) -> Result<(u8, u8, u8), &'static str> {
    /* Iteration not working :/ */
    for bus in 0u8..=255u8 {
        for slot in 0u8..32u8 {
            /* If the device at the given bus and slot # is unvalid, just continue */
            if pci_get_vendor_id(bus, slot, 0x00) == 0xFFFF {
                continue;
            }

            /* Return + Exit if the device at the first function matches the class
            and subclass */
            let target_class: u8 = pci_get_class(bus, slot, 0x00);
            let target_subclass: u8 = pci_get_subclass(bus, slot, 0x00);
            if target_class == class && target_subclass == subclass {
                return Ok((bus, slot, 0x00));
            }

            /* If the bit 7 is not set, the device has NOT multiple functions */
            if pci_get_header_type(bus, slot, 0x00) & 0x80 == 0x00 {
                continue;
            }

            /* Otherwise check through the device's other functions for the given class
            and subclass code */
            for function in 1u8..8u8 {
                let target_class: u8 = pci_get_class(bus, slot, function);
                let target_subclass: u8 = pci_get_subclass(bus, slot, function);

                /* If found at different function number, return and exist */
                if target_class == class && target_subclass == subclass {
                    return Ok((bus, slot, function));
                }
            }
        }
    }

    /* If iterated through all the devices and not found */
    Err("Device was not found")
}
