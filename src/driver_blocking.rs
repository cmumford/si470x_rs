use embedded_hal::i2c::{I2c, SevenBitAddress};

use super::driver_common::{ChipInfo, Register, Si470xError};

const SI470X_I2C_ADDRESS: SevenBitAddress = 0x10;

pub struct Si470x<I2C> {
    i2c: I2C,
}

impl<I2C> Si470x<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn read_register(&mut self, reg: Register) -> Result<u16, Si470xError<I2C::Error>> {
        let mut buffer = [0u8; 2];
        let reg_byte: u8 = reg.into();

        self.i2c
            .write_read(SI470X_I2C_ADDRESS, &[reg_byte], &mut buffer)
            .map_err(Si470xError::I2c)?;

        Ok(u16::from_be_bytes(buffer))
    }

    pub fn write_register(
        &mut self,
        reg: Register,
        value: u16,
    ) -> Result<(), Si470xError<I2C::Error>> {
        let bytes = value.to_be_bytes();
        let buf = [reg.into(), bytes[0], bytes[1]];

        self.i2c
            .write(SI470X_I2C_ADDRESS, &buf)
            .map_err(Si470xError::I2c)?;

        Ok(())
    }

    pub fn get_chip_info(&mut self) -> Result<ChipInfo, Si470xError<I2C::Error>> {
        let reg_val = self.read_register(Register::ChipId)?;
        Ok(ChipInfo {
            revision: ((reg_val >> 10) & 0x3F) as u8,
            device: ((reg_val >> 6) & 0x0F) as u8,
            firmware: (reg_val & 0x3F) as u8,
        })
    }

    pub fn ping(&mut self) -> Result<(), Si470xError<I2C::Error>> {
        self.i2c
            .write(SI470X_I2C_ADDRESS, &[])
            .map_err(Si470xError::I2c)
    }
}
