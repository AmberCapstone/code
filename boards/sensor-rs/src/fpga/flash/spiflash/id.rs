#[derive(Debug, PartialEq)]
pub struct Id {
    pub manufacturer: u8,
    pub memory_type: u8,
    pub capacity: u8,
}

impl Id {
    #[cfg(feature = "nucleo")]
    pub const fn expected() -> Self {
        Self {
            manufacturer: 0x20,
            memory_type: 0x80,
            capacity: 0x11,
        }
    }

    #[cfg(not(feature = "nucleo"))]
    pub const fn expected() -> Self {
        Self {
            manufacturer: 0xef,
            memory_type: 0x40,
            capacity: 0x14,
        }
    }
}

impl From<&[u8; 3]> for Id {
    fn from(value: &[u8; 3]) -> Self {
        Self {
            manufacturer: value[0],
            memory_type: value[1],
            capacity: value[2],
        }
    }
}
