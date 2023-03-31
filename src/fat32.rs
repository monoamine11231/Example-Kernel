use std::fs::File;
use std::io::{Read};

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
    large_sector_count: u32
}

pub fn read_bootsector(fd: &mut File) -> BootSector {
    let mut buffer = [0 as u8;36];
    /* Replace later with the actual disk read implementation */
    fd.read_exact(&mut buffer).unwrap();
    let o: BootSector = unsafe { *(buffer.as_ptr() as *const _) };
    return o;
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

