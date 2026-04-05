#![allow(unused)]

pub use sccb::Reg;

pub(super) const ADDRESS: u8 = 0x21;

pub struct CameraInterface<I2C: embedded_hal_async::i2c::I2c> {
    i2c: I2C,
}

impl<I2C: embedded_hal_async::i2c::I2c> CameraInterface<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub async fn read_register(&mut self, reg: Reg) -> Result<u8, I2C::Error> {
        let mut out = [0u8];
        self.i2c.write(ADDRESS, &[reg as u8]).await?;
        self.i2c.read(ADDRESS, &mut out).await?;
        Ok(out[0])
    }

    pub async fn write_register(&mut self, reg: Reg, data: u8) -> Result<(), I2C::Error> {
        self.i2c.write(ADDRESS, &[reg as u8, data]).await
    }

    pub async fn modify_register(&mut self, reg: Reg, modifier: impl FnOnce(u8) -> u8) -> Result<(), I2C::Error> {
        let old_val = self.read_register(reg).await?;
        let new_val = modifier(old_val);
        self.write_register(reg, new_val).await
    }
}
