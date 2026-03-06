#[derive(Debug, PartialEq)]
pub struct Id {
    pub manufacturer_id: u8,
    pub memory_type: u8,
    pub capacity: u8,
}

impl Id {
    #[cfg(not(feature = "pcb"))]
    pub const fn expected() -> Self {
        Self {
            manufacturer_id: 0x20,
            memory_type: 0x80,
            capacity: 0x11,
        }
    }

    #[cfg(feature = "pcb")]
    pub const fn expected() -> Self {
        Self {
            manufacturer_id: 0x3f,
            memory_type: 0x40,
            capacity: 0x14,
        }
    }
}

impl From<&[u8; 3]> for Id {
    fn from(value: &[u8; 3]) -> Self {
        Self {
            manufacturer_id: value[0],
            memory_type: value[1],
            capacity: value[2],
        }
    }
}
