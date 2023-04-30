use heapless::String;

use crate::drivers::ide::{self, ATADirection, IDE};
use crate::mem::memory::kmemcpy;
use crate::tooling::qemu_io::{qemu_print, qemu_print_hex, qemu_println};

/* Temporary until proper memory allocation is fixed */
const LOAD_ADDR: u64 = 0x43000000;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
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
#[derive(Debug, Copy, Clone)]
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
#[derive(Debug, Copy, Clone)]
pub struct FSInfoMain {
    signature_1: u32,
    reserved: [u8; 480],
    signature_2: u32,
    last_free_cluster: u32,
    start_cluster: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
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

#[repr(C, align(32))]
#[derive(Default, Debug, Copy, Clone)]
pub struct ShortDirectoryEntry {
    file_name: [u8; 8],
    file_ext: [u8; 3],
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

#[repr(C, align(32))]
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

    /// Filename given with extension in point form
    pub fn compare_filename(&self, filename: &str) -> Result<bool, &'static str> {
        let mut part_counter: u8 = 0x00;
        for part in filename.split('.') {
            match part_counter {
                0x00 => {
                    let mut s: String<8> = String::new();

                    for c in self
                        .short
                        .file_name
                        .iter()
                        .filter(|x| x.ne(&&0x20))
                        .into_iter()
                    {
                        s.push(*c as char);
                    }

                    if s.as_str().cmp(part) != core::cmp::Ordering::Equal {
                        return Ok(false);
                    }
                }
                0x01 => {
                    let mut s: String<3> = String::new();

                    for c in self
                        .short
                        .file_ext
                        .iter()
                        .filter(|x| x.ne(&&0x20))
                        .into_iter()
                    {
                        s.push(*c as char);
                    }

                    if s.as_str().cmp(part) != core::cmp::Ordering::Equal {
                        return Ok(false);
                    }
                }
                _ => return Err("Given filename is not valid!"),
            }

            part_counter += 1;
        }

        /* If filename is without extension, don't forget to check if the extension
         * is empty as well */
        if part_counter == 1 {
            let mut s: String<3> = String::new();

            for c in self
                .short
                .file_ext
                .iter()
                .filter(|x| x.ne(&&0x20))
                .into_iter()
            {
                s.push(*c as char);
            }

            if s.len() != 0x00 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn chain(&self) -> u32 {
        let mut chain: u32 = 0x00000000;
        chain = (self.short.high_first_cluster as u32) << 16;
        chain |= self.short.low_first_entry_cluster as u32;

        return chain;
    }
}

pub enum FATEntry {}

impl FATEntry {
    #[inline]
    pub fn end(entry: u32) -> bool {
        return entry >= 0x0FFFFFF8;
    }
}

pub struct FAT {
    iter_offset: u32,

    fat_address: u64,
    sectors_per_fat: u32,
    fat_num: u8,
}

impl FAT {
    pub fn new(fat_address: u64, sectors_per_fat: u32, fat_num: u8) -> Self {
        Self {
            iter_offset: 0x02,

            fat_address: fat_address,
            sectors_per_fat: sectors_per_fat,
            fat_num: fat_num,
        }
    }

    pub fn reset_iter(&mut self) {
        self.iter_offset = 0x02;
    }
}

impl Iterator for FAT {
    type Item = u32;

    /* Returns empty chains in FAT */
    fn next(&mut self) -> Option<Self::Item> {
        let fat: *mut u32 = self.fat_address as *mut u32;
        while unsafe { *fat.offset(self.iter_offset as isize) } != 0x01 {
            self.iter_offset += 1;
        }

        let r: Option<Self::Item> = Some(self.iter_offset);
        self.iter_offset += 1;
        return r;
    }
}

pub struct FAT32<'a> {
    ide_processor: &'a mut IDE,
    fat_processor: FAT,

    partition_start_lba: u32,
    reserved_sectors: u16,
    sectors_per_cluster: u8,
    root_dir_num: u16,
    total_sectors: u32,
    bytes_per_sector: u16,
    root_dir_cluster: u32,
}

impl<'a> FAT32<'a> {
    pub fn new(ide_processor: &'a mut IDE) -> Result<Self, &'static str> {
        ide_processor.ata_access_pio(ATADirection::Read, 0, 0x01, 1, 0x41000000);

        let bootsector: BootSector = unsafe { *(0x41000000 as *const _) };
        let extended_boot_record: &ExtendedBootRecord = unsafe { &*(0x41000024 as *const _) };
        if extended_boot_record.signature != 0x28 && extended_boot_record.signature != 0x29 {
            return Err("Signature in extended boot record is not valid!");
        }

        /* Check byte by byte, since the u64 is misaligned :( */
        let system_identifier_match: [u8; 8] = [0x46, 0x41, 0x54, 0x33, 0x32, 0x20, 0x20, 0x20];
        for i in 0usize..8usize {
            if system_identifier_match[i] != extended_boot_record.system_identifier[i] {
                return Err("System identifier string in extended boot record is not valid!");
            }
        }

        let fsinfo_address: u64 =
            0x41000000 + 0x200 * extended_boot_record.fsinfo_sector_num as u64;
        ide_processor.ata_access_pio(
            ATADirection::Read,
            0,
            bootsector.num_hidden_sectors as u64 + extended_boot_record.fsinfo_sector_num as u64,
            1,
            fsinfo_address,
        );

        let fsinfo: &FSInfoMain = unsafe { &*(fsinfo_address as *const _) };
        if fsinfo.signature_1 != 0x41615252 || fsinfo.signature_2 != 0x61417272 {
            return Err("Signatures in FSInfo struct are not valid!");
        }

        let mut total_sectors: u32 = bootsector.num_sectors as u32;
        if total_sectors == 0x00 {
            total_sectors = bootsector.large_sector_count;
        }

        /* Load FAT */
        ide_processor.ata_access_pio(
            ATADirection::Read,
            0,
            bootsector.num_hidden_sectors as u64 + bootsector.reserved_sectors as u64,
            (extended_boot_record.sectors_per_fat * bootsector.num_fats as u32) as u8,
            0x42000000,
        );

        Ok(Self {
            ide_processor: ide_processor,
            fat_processor: FAT::new(
                0x42000000,
                extended_boot_record.sectors_per_fat,
                bootsector.num_fats,
            ),
            partition_start_lba: bootsector.num_hidden_sectors,
            reserved_sectors: bootsector.reserved_sectors,
            sectors_per_cluster: bootsector.sectors_per_cluster,
            root_dir_num: bootsector.num_root_dir_entries,
            total_sectors: total_sectors,
            bytes_per_sector: bootsector.bytes_per_sector,
            root_dir_cluster: extended_boot_record.cluster_num_root_dir,
        })
    }

    pub fn read_file(&mut self, path: &str, to: &mut [u8], n: usize) -> Result<(), &'static str> {
        let packed: Option<(DirectoryEntry, u32, u64)> = self.traverse(path)?;
        if packed.is_none() {
            return Err("File was not found!");
        }

        let unpacked: (DirectoryEntry, u32, u64) = packed.unwrap();
        let entry: DirectoryEntry = unpacked.0;
        if entry.short.file_attribute == 0x10 {
            return Err("File is a directory!");
        }

        /* main FAT where we are searching clusters */
        let fat: *const u32 = self.fat_processor.fat_address as *const u32;
        let mut current_chain: u32 = entry.chain();
        let mut current_read: u32 = 0x00;

        let mut remaining: usize = core::cmp::min(entry.short.file_size as usize, n);
        /* Cluster in bytes */
        let clb: usize = self.bytes_per_sector as usize * self.sectors_per_cluster as usize;
        while !FATEntry::end(current_chain) {
            self.ide_processor.ata_access_pio(
                ATADirection::Read,
                0,
                self.cluster_lba(current_chain as usize),
                self.sectors_per_cluster,
                LOAD_ADDR,
            );

            let offset_in_mem: u64 = entry.short.file_size as u64 - remaining as u64;
            if remaining < clb {
                kmemcpy(
                    unsafe { (LOAD_ADDR + offset_in_mem) as *const u8 },
                    unsafe { to.as_mut_ptr() },
                    remaining,
                );
                break;
            }

            /* Copy one cluster */
            kmemcpy(
                unsafe { (LOAD_ADDR + offset_in_mem) as *const u8 },
                unsafe { to.as_mut_ptr() },
                clb,
            );
            remaining -= clb;
            current_chain = unsafe { *fat.offset(current_chain as isize) } & 0x0FFFFFFF;
        }
        Ok(())
    }

    /// Deletes a file on given path. Returns error if operation was not performed 
    pub fn delete_file(&mut self, path: &str) -> Result<(), &'static str> {
        let packed: Option<(DirectoryEntry, u32, u64)> = self.traverse(path)?;
        if packed.is_none() {
            return Err("File was not found!");
        }

        let unpacked: (DirectoryEntry, u32, u64) = packed.unwrap();
        let entry: DirectoryEntry = unpacked.0;
        if entry.short.file_attribute == 0x10 {
            return Err("File is a directory!");
        }

        Ok(self.delete_object(&unpacked.0, unpacked.1, unpacked.2)?)
    }

    /// Deletes a directory on given path. Returns error if operation was not performed 
    pub fn delete_directory(&mut self, path: &str) -> Result<(), &'static str> {
        let packed: Option<(DirectoryEntry, u32, u64)> = self.traverse(path)?;
        if packed.is_none() {
            return Err("Directory was not found!");
        }

        let unpacked: (DirectoryEntry, u32, u64) = packed.unwrap();
        let entry: DirectoryEntry = unpacked.0;
        if entry.short.file_attribute != 0x10 {
            return Err("File is a file!");
        }

        Ok(self.delete_object(&unpacked.0, unpacked.1, unpacked.2)?)
    }

    /// Deletes an object with given directory entry `entry`, on cluster # `cluster` and
    /// with an offset inside of the given cluster #, of `offset`
    fn delete_object(
        &mut self,
        entry: &DirectoryEntry,
        cluster: u32,
        offset: u64,
    ) -> Result<(), &'static str> {
        /* The LBA address of the sector where the given directory entry is found */
        let lba: u64 = self.cluster_lba(cluster as usize) + offset / self.bytes_per_sector as u64;
        let offset_in_sector: u64 = offset % self.bytes_per_sector as u64;

        /* Read that sector, modify the entry and write back to disk... */
        self.ide_processor
            .ata_access_pio(ATADirection::Read, 0x00, lba, 1, LOAD_ADDR);

        unsafe {
            /* Mark directory entry as unused by setting the first byte to 0xE5 */
            let addr: *mut u8 = LOAD_ADDR as *mut u8;
            *addr.offset(offset_in_sector as isize) = 0xE5;
        }

        /* Write the modified directory entry back to disk */
        self.ide_processor
            .ata_access_pio(ATADirection::Write, 0x00, lba, 1, LOAD_ADDR);

        /* Clean the cluster chain in FAT, that is associated with the given object */
        self.deallocate_chain(entry.chain());

        /* Write the FATs to the disk */
        self.sync_fat();
        Ok(())
    }

    /// Search of an object (directory or file) with name `name` in the directory chain
    /// with chain #, `chain` 
    fn search_in_dir(
        &mut self,
        chain: u32,
        name: &str,
    ) -> Result<Option<(DirectoryEntry, u32, u64)>, &'static str> {
        /* main FAT where we are searching clusters */
        let fat: *const u32 = self.fat_processor.fat_address as *const u32;
        let mut current_chain: u32 = chain;
        while !FATEntry::end(current_chain) {
            self.ide_processor.ata_access_pio(
                ATADirection::Read,
                0,
                self.cluster_lba(current_chain as usize),
                self.sectors_per_cluster,
                LOAD_ADDR,
            );

            let mut dir_offset: u64 = LOAD_ADDR;
            while let Some((entry, size, skip)) = DirectoryEntry::fetch(dir_offset) {
                /* Deleted entry or long name entry - move on */
                if skip || entry.short.file_attribute == 0x0F {
                    dir_offset += size as u64;
                    continue;
                }

                if entry.compare_filename(name)? {
                    return Ok(Some((entry, current_chain, dir_offset - LOAD_ADDR)));
                }
                dir_offset += size as u64;
            }

            current_chain = unsafe { *fat.offset(current_chain as isize) } & 0x0FFFFFFF;
        }
        Ok(None)
    }

    /// Traverses the path and returns the parsed directory entry together with the
    /// cluster number and offset in the cluster, if the object was found in path
    pub fn traverse(
        &mut self,
        path: &str,
    ) -> Result<Option<(DirectoryEntry, u32, u64)>, &'static str> {
        let mut found_entry: Option<(DirectoryEntry, u32, u64)> = None;

        let mut current_chain: u32 = 0x02;
        for part in path.split('/') {
            /* If a directory entry was found, unpack it and search through it */
            if let Some(entry) = found_entry {
                if entry.0.short.file_attribute != 0x10 {
                    return Err("Trying to search through a file!");
                }

                /* Assemble the cluster number where the directory is placed */
                current_chain = entry.0.chain();
            }

            found_entry = self.search_in_dir(current_chain, part)?;
            if found_entry.is_none() {
                break;
            }
        }

        Ok(found_entry)
    }

    pub fn cluster_lba(&self, cluster: usize) -> u64 {
        self.partition_start_lba as u64
            + self.reserved_sectors as u64
            + (self.fat_processor.fat_num as u64) * (self.fat_processor.sectors_per_fat as u64)
            + (cluster as u64 - 2) * (self.sectors_per_cluster as u64)
    }

    /// Writes the FAT tables in memory to all FATS on the disk
    fn sync_fat(&mut self) {
        /* Write back all FATS */
        for i in 0..self.fat_processor.fat_num {
            self.ide_processor.ata_access_pio(
                ATADirection::Write,
                0x00,
                self.partition_start_lba as u64
                    + self.reserved_sectors as u64
                    + (self.fat_processor.sectors_per_fat * i as u32) as u64,
                self.fat_processor.sectors_per_fat as u8,
                self.fat_processor.fat_address,
            )
        }
    }

    /// Creates a cluster chain by searching unused fat entries on FAT
    fn allocate_chain(&mut self, length: usize) -> u32 {
        self.fat_processor.reset_iter();

        /* main FAT where we are searching clusters */
        let fat: *mut u32 = self.fat_processor.fat_address as *mut u32;

        /* Find a new empty entry */
        let mut start: u32 = self.fat_processor.next().unwrap();

        let mut previous: u32 = start;
        for i in 1..length {
            let current: u32 = self.fat_processor.next().unwrap();
            unsafe {
                *fat.offset(previous as isize) = current;
            }

            previous = current;
        }

        /* Mark last chain as last */
        unsafe {
            *fat.offset(previous as isize) = 0xFFFFFFFF;
        }
        return start;
    }

    /// Marks the whole cluster chain as unused. `start` is the beginning of the cluster
    fn deallocate_chain(&self, start: u32) {
        /* main FAT where we are searching clusters */
        let fat: *mut u32 = self.fat_processor.fat_address as *mut u32;

        let mut current_chain: u32 = start;
        while !FATEntry::end(current_chain) {
            unsafe {
                let new: u32 = *fat.offset(current_chain as isize);
                /* Mark as empty */
                *fat.offset(current_chain as isize) = 0x00000000;

                current_chain = new;
            }
        }
    }
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
