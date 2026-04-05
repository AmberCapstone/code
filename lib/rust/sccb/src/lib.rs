#![cfg_attr(feature = "no-std", no_std)]

mod reg;
pub use reg::Reg;

pub const SENSOR_HEIGHT: u16 = 480;
pub const SENSOR_WIDTH: u16 = 640;
pub const BLANK_COLUMNS: u16 = 144;
pub const BLANK_ROWS: u16 = 30;

const MAX_REGISTER: usize = 0xC9;
pub const NUM_REGISTERS: usize = MAX_REGISTER + 1;
