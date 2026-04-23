#[repr(u8)]
pub enum Command {
    WriteEnable = 0x06,
    WriteDisable = 0x04,
    ReadIdentification = 0x9f,
    ReadStatusRegister = 0x05,
    WriteStatusRegister = 0x01,
    WriteToLockRegister = 0xe5,
    ReadLockRegister = 0xe8,
    ReadDataBytes = 0x03,
    ReadDataBytesHighSpeed = 0x0b,
    PageProgram = 0x02,
    SubsectorErase = 0x20,
    SectorErase = 0xd8,
    ChipErase = 0xc7,
    DeepPowerDown = 0xb9,
    ReleaseDeepPowerDown = 0xab,
}
