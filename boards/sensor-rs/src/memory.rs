// Bring the memory layout defined in memory.x into Rust

use core::ptr::addr_of;

// These values are defined by the linker
unsafe extern "C" {
    static __flash_start: u32;
    static __flash_end: u32;
    static __nvm_start: u32;
    static __nvm_end: u32;
}

#[allow(unused, reason = "Struct should completely describe memory.x")]
pub struct Map {
    pub flash_start: u32,
    pub flash_end: u32,
    pub nvm_start: u32,
    pub nvm_end: u32,
}

pub const PAGE_SIZE: u32 = 0x400;

impl Map {
    pub fn get() -> Self {
        Self {
            flash_start: addr_of!(__flash_start) as u32,
            flash_end: addr_of!(__flash_end) as u32,
            nvm_start: addr_of!(__nvm_start) as u32,
            nvm_end: addr_of!(__nvm_end) as u32,
        }
    }
}
