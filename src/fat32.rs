use crate::drivers::ide::{self, ATADirection, IDE};
use crate::tooling::qemu_io::{qemu_print_hex, qemu_println};

#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct BootSector {
    /* All numbers are in little endian */
    boot_jmp: [u8; 3],
    oem_identifier: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    num_root_dir_entries: u16,
    num_sectors: u16,
    media_description_type: u8,
    num_sectors_per_fat: u16,
    num_sectors_per_track: u16,
    num_heads_on_storage: u16,
    num_hidden_sectors: u32,
    large_sector_count: u32,
}

#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct ExtendedBootRecord {
    sectors_per_fat: u32,
    flags: u16,
    fat_version_major: u8,
    fat_version_minor: u8,
    cluster_num_root_dir: u32,
    fsinfo_sector_num: u16,
    backup_sector_num: u16,
    reserved: [u8; 12],
    drive_num: u8,
    flags_windows_nt: u8,
    signature: u8,
    volume_id_serial_num: u32,
    volume_label_string: [u8; 11],
    system_identifier: [u8; 8],
}

/* The part of FSInfo struct after the 480 reserved bytes */
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct FSInfoMain {
    signature_2: u32,
    last_free_cluster: u32,
    start_cluster: u32,
}

#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct LongDirectoryEntry {
    order: u8,
    first_u8: [u8; 10],
    attribute: u8,
    long_entry_type: u8,
    checksum: u8,
    second_u8: [u8; 12],
    reserved: u16,
    third_u8: [u8; 4],
}

#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct ShortDirectoryEntry {
    file_name: [u8; 11],
    file_attribute: u8,
    reserved_windows_nt: u8,
    creation_time_100ms: u8,
    creation_time: u16,
    creation_date: u16,
    last_accessed_date: u16,
    high_first_cluster: u16,
    last_modification_time: u16,
    last_modification_date: u16,
    low_first_entry_cluster: u16,
    file_size: u32,
}

#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone)]
pub struct DirectoryEntry {
    short: ShortDirectoryEntry,
}

impl DirectoryEntry {
    /* Fetch DirectoryEntry from given address and return size read and if it needs to
     * be skipped (unused) */
    pub fn fetch(from: u64) -> Option<(Self, u8, bool)> {
        unsafe {
            /* If nonvalid */
            if *(from as *const u8) == 0x00 {
                return None;
            }

            let skip: bool = *(from as *const u8) == 0xE5;

            let mut short: ShortDirectoryEntry;

            let mut size: u8 = 0;

            /* Long entry */
            // if *((from + 11) as *const u8) == 0x0F {
            //     long = Some(*(from as *const LongDirectoryEntry));
            //     size += 32;
            // }

            short = *((from + size as u64) as *const ShortDirectoryEntry);
            size += 32;

            return Some((Self { short: short }, size, skip));
        };
    }
}

pub struct FAT32<'a> {
    ide_processor: &'a mut IDE,
    fat_address: u64,

    partition_start_lba: u32,
    reserved_sectors: u16,
    sectors_per_cluster: u8,
    fat_num: u8,
    root_dir_num: u16,
    total_sectors: u32,
    bytes_per_sector: u16,
    root_dir_cluster: u32,
    sectors_per_fat: u32,
}

impl<'a> FAT32<'a> {
    pub fn new(ide_processor: &'a mut IDE) -> Self {
        let structs = read_fat32_structs(ide_processor).unwrap();

        let mut total_sectors: u32 = structs.0.num_sectors as u32;
        if total_sectors == 0x00 {
            total_sectors = structs.0.large_sector_count;
        }

        /* Load FAT */
        ide_processor.ata_access_pio(
            ATADirection::Read,
            0,
            structs.0.num_hidden_sectors as u64 + structs.0.reserved_sectors as u64,
            (structs.1.sectors_per_fat * structs.0.num_fats as u32) as u8,
            0x41000000,
        );

        Self {
            ide_processor: ide_processor,
            fat_address: 0x41000000,

            partition_start_lba: structs.0.num_hidden_sectors,
            reserved_sectors: structs.0.reserved_sectors,
            sectors_per_cluster: structs.0.sectors_per_cluster,
            fat_num: structs.0.num_fats,
            root_dir_num: structs.0.num_root_dir_entries,
            total_sectors: total_sectors,
            bytes_per_sector: structs.0.bytes_per_sector,
            root_dir_cluster: structs.1.cluster_num_root_dir,
            sectors_per_fat: structs.1.sectors_per_fat,
        }
    }

    pub fn find_dir_entry_by_filename(
        &mut self,
        filename: &str,
    ) -> Result<DirectoryEntry, &'static str> {
        const LOAD_ADDR: u64 = 0x41500000;

        /* 0x02 is the first valid cluster! */
        let mut cluster_counter: usize = 0x02;
        self.ide_processor.ata_access_pio(
            ATADirection::Read,
            0,
            self.cluster_lba(cluster_counter),
            self.sectors_per_cluster,
            LOAD_ADDR,
        );

        let mut dir_offset: u64 = LOAD_ADDR;
        let dir_end: u64 =
            LOAD_ADDR + (self.sectors_per_cluster as u64) * self.bytes_per_sector as u64;
        /* If first byte is 0 in directory, it indicates the end of root directory */
        unsafe {
            while let Some((entry, size, skip)) = DirectoryEntry::fetch(dir_offset) {
                dir_offset += size as u64;
                /* Unused entry */
                if skip {
                    continue;
                }

                /* If not done, load one more cluster */
                if dir_offset >= dir_end {
                    cluster_counter += 1;
                    self.ide_processor.ata_access_pio(
                        ATADirection::Read,
                        0,
                        self.cluster_lba(cluster_counter),
                        self.sectors_per_cluster,
                        LOAD_ADDR,
                    );
                    dir_offset = LOAD_ADDR;
                }
            }
        }

        Err("asdasdas")
    }

    pub fn cluster_lba(&self, cluster: usize) -> u64 {
        self.partition_start_lba as u64
            + self.reserved_sectors as u64
            + (self.fat_num as u64) * (self.sectors_per_fat as u64)
            + (cluster as u64 - 2) * (self.sectors_per_cluster as u64)
    }
}

fn read_fat32_structs(
    ide_processor: &mut IDE,
) -> Result<(BootSector, ExtendedBootRecord, FSInfoMain), &'static str> {
    ide_processor.ata_access_pio(ATADirection::Read, 0, 0x01, 2, 0x41000000);

    let bootsector: BootSector = unsafe { *(0x41000000 as *const _) };
    let extended_boot_record: ExtendedBootRecord = unsafe { *(0x41000024 as *const _) };
    if extended_boot_record.signature != 0x28 && extended_boot_record.signature != 0x29 {
        return Err("Signature in extended boot record is not valid!");
    }

    if unsafe { *(extended_boot_record.system_identifier.as_ptr() as *const u64) }
        != 0x2020203233544146
    {
        return Err("System identifier string in extended boot record is not valid!");
    }

    let fsinfo_address: u64 = 0x41000000 + 0x200 * extended_boot_record.fsinfo_sector_num as u64;
    if unsafe { *(fsinfo_address as *const u32) } != 0x41615252 {
        return Err("Lead signature in FSInfo struct is not valid!");
    }

    let fsinfo: FSInfoMain = unsafe { *((fsinfo_address + 0x1E4) as *const _) };
    if fsinfo.signature_2 != 0x61417272 {
        return Err("Second signature in FSInfo struct is not valud!");
    }

    Ok((bootsector, extended_boot_record, fsinfo))
}

#[cfg(test)]
mod fat32_tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_read_bootsector_data_is_ordered() {
        let mut f = File::open("src/test_files/unvalid_with_bootsector.img").unwrap();
        let boot_sector = read_bootsector(&mut f);

        /* Tests that the bootsector loaded is readed correctly. The image was generated
         * in C.
         */
        assert!(boot_sector.boot_jmp == [0x00, 0x01, 0x02]);
        assert!(boot_sector.oem_identifier == [0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A]);
        assert!(boot_sector.bytes_per_sector == 0x0B);
        assert!(boot_sector.sectors_per_cluster == 0x0C);
        assert!(boot_sector.reserved_sectors == 0x0D);
        assert!(boot_sector.num_fats == 0x0E);
        assert!(boot_sector.num_root_dir_entries == 0x0F);
        assert!(boot_sector.num_sectors == 0x10);
        assert!(boot_sector.media_description_type == 0x11);
        assert!(boot_sector.num_sectors_per_fat == 0x12);
        assert!(boot_sector.num_sectors_per_track == 0x13);
        assert!(boot_sector.num_heads_on_storage == 0x14);
        assert!(boot_sector.num_hidden_sectors == 0x15);
        assert!(boot_sector.large_sector_count == 0x16);
    }
}
