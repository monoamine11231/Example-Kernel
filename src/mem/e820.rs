/* This struct is not being used, but take it as a reference for development */
#[repr(u32)]
enum RegionType {
    USABLE = 0x00,
    RESERVED = 0x01,
    ACPI_RECLAIMABLE = 0x02,
    ACPI_NVS = 0x03,
    BAD_MEM = 0x04,
}

#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct E820 {
    addr_base: u64,
    addr_length: u64,
    region_type: u32,
    acpi_ext: u32, /* Doesn't always exist */
}

fn how_to_use() {
    let entries_num: u32 = *(0x7e00 as *const u32);
    let entries = 0x7e00 as *const E820;

    /* Hopefully so */
    entries.offset(0);
}
