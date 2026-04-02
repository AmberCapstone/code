#![allow(unused)]

#[repr(u8)]
pub enum Command {
    Write = 0x01,
    Read = 0x02,
    FakeCaptureWrite = 0x03,
    FakeCaptureVga = 0x04,
    Reset = 0x00,
    RealCapture = 0x05,
}
