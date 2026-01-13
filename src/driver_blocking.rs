use super::driver_common::*;
use embedded_hal::i2c::I2c;

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

    pub fn read_register_bytes(
        &mut self,
        reg: Register,
    ) -> Result<[u8; 2], Si470xError<I2C::Error>> {
        let mut buffer = [0u8; 2];
        let reg_byte: u8 = reg.into();

        self.i2c
            .write_read(SI470X_I2C_ADDRESS, &[reg_byte], &mut buffer)
            .map_err(Si470xError::I2c)?;

        Ok(buffer)
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

    pub fn write_register_bytes(
        &mut self,
        reg: Register,
        value_bytes: [u8; 2],
    ) -> Result<(), Si470xError<I2C::Error>> {
        let reg_byte: u8 = reg.into();
        let buf = [reg_byte, value_bytes[0], value_bytes[1]];

        self.i2c
            .write(SI470X_I2C_ADDRESS, &buf)
            .map_err(Si470xError::I2c)?;

        Ok(())
    }

    // Enable or disable the device. Before disabling RDS should be disabled
    // according to the datasheet.
    pub fn set_enable(&mut self, enable: bool) -> Result<(), Si470xError<I2C::Error>> {
        let mut reg = self.read_register(Register::PowerCfg)?;
        // Note: Datasheet says "The ENABLE bit should never be written to a 0".
        if enable {
            reg = reg.set(PowerCfg::ENABLE).clear(PowerCfg::DISABLE);
        } else {
            reg = reg.set(PowerCfg::ENABLE).set(PowerCfg::DISABLE);
        }
        self.write_register(Register::PowerCfg, reg)
    }

    pub fn set_volume(&mut self, volume: u8) -> Result<(), Si470xError<I2C::Error>> {
        let reg = self.read_register_bytes(Register::SysConfig2)?;
        let mut config = SysConfig2::from_bytes(reg);
        config.set_volume(volume);
        self.write_register_bytes(Register::SysConfig2, config.into())
    }

    pub fn set_channel_spacing(
        &mut self,
        channel_spacing: ChannelSpacing,
    ) -> Result<(), Si470xError<I2C::Error>> {
        let reg = self.read_register_bytes(Register::SysConfig2)?;
        let mut config = SysConfig2::from_bytes(reg);
        config.set_space(channel_spacing);
        self.write_register_bytes(Register::SysConfig2, config.into())
    }

    pub fn set_oscillator_enable(&mut self, enable: bool) -> Result<(), Si470xError<I2C::Error>> {
        let reg = self.read_register_bytes(Register::Test1)?;
        let mut config = Test1::from_bytes(reg);
        config.set_xoscen(enable);
        self.write_register_bytes(Register::Test1, config.into())
    }

    pub fn get_chip_info(&mut self) -> Result<ChipInfo, Si470xError<I2C::Error>> {
        let reg_val = self.read_register(Register::ChipId)?;
        Ok(ChipInfo {
            revision: (reg_val >> 10) as u8,
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
