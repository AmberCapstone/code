#![allow(unused)]

#[repr(u8)]
pub enum Command {
    Reset = 0x00,
    Write = 0x01,
    ReadData = 0x02,
    FakeCaptureWrite = 0x03,
    FakeCaptureVga = 0x04,
    RealCapture = 0x05,
    GetVessels = 0x06, // return [num, x1l, x1h, y1l, y1h..., x5l, x5h, y5l, y5h] - 21 bytes
}
