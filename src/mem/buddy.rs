#[repr(u8)]
enum Flag {
    Free,
    Used,
}

struct ChunkMetaData {
    pub chunk_flag: Flag,
    pub order: i16,
    pub next: *mut ChunkMetaData,
    pub prev: *mut ChunkMetaData,
}

pub fn free(ptr: *mut u8) {}

pub fn alloc() {}
