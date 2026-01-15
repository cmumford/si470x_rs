#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c as AsyncI2c;

use super::driver_common::*;

pub struct Si470x<I2C> {
    i2c: I2C,
}

#[cfg(feature = "async")]
impl<I2C> Si470x<I2C>
where
    I2C: AsyncI2c,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub async fn read_all_registers(&mut self) -> Result<[u8; 32], Si470xError<I2C::Error>> {
        let mut registers = [0u8; 32];

        // "For read operations, the device acknowledge is followed by an eight
        // bit data word shifted out on falling SCLK edges. An internal address
        // counter automatically increments to allow continuous data byte
        // reads, starting with the upper byte of register 0Ah, followed by the
        // lower byte of register 0Ah, and onward until the lower byte of the
        // last register is reached. The internal address counter then
        // automatically wraps around to the upperbyte of register 00h and
        // proceeds from there until continuous reads cease."

        // See command above ReadRegIdx for order of data.
        self.i2c
            .read(SI470X_I2C_ADDRESS, &mut registers)
            .await
            .map_err(Si470xError::I2c)?;
        Ok(registers)
    }

    // Write all "writable" registers (02h through 07h).
    // `registers` is in the read order defined by ReadRegIdx.
    pub async fn write_all_registers(
        &mut self,
        registers: [u8; 32],
    ) -> Result<(), Si470xError<I2C::Error>> {
        // "An internal address counter automatically increments to allow
        // continuous data byte writes, starting with the upper byte of register
        // 02h, followed by the lower byte of register 02h, and onward until
        // the lower byte of the last register is reached. The internal address
        // counter then automatically wraps around to the upper byte of
        // register 00h and proceeds from there until continuous writes end."
        const START_IDX: usize = 2 * ReadRegIdx::PowerCfg as usize;
        const END_IDX: usize = 2 * ReadRegIdx::Test1 as usize;

        self.i2c
            .write(SI470X_I2C_ADDRESS, &registers[START_IDX..END_IDX + 2])
            .await
            .map_err(Si470xError::I2c)?;

        Ok(())
    }

    // Enable or disable the device. Before disabling RDS should be disabled
    // according to the datasheet.
    pub async fn set_enable(&mut self, enable: bool) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().await.unwrap();
        let idx = 2 * ReadRegIdx::PowerCfg as usize;
        let mut config = PowerCfg::from_bytes([registers[idx], registers[idx + 1]]);

        // Note: Datasheet says "The ENABLE bit should never be written to a 0".
        config.set_enable(true);
        config.set_disable(!enable);

        registers[idx..idx + 2].copy_from_slice(&config.into_bytes());
        self.write_all_registers(registers).await
    }

    pub async fn set_mute(&mut self, muted: bool) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().await.unwrap();
        let idx = 2 * ReadRegIdx::PowerCfg as usize;
        let mut config = PowerCfg::from_bytes([registers[idx], registers[idx + 1]]);

        config.set_dmute(muted);

        registers[idx..idx + 2].copy_from_slice(&config.into_bytes());
        self.write_all_registers(registers).await
    }

    // Set the radio volume. Volume is 4-bit unsigned.
    pub async fn set_volume(&mut self, volume: u8) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().await.unwrap();
        let idx = 2 * ReadRegIdx::SysConfig2 as usize;
        let mut config = SysConfig2::from_bytes([registers[idx], registers[idx + 1]]);

        config.set_volume(volume);

        registers[idx..idx + 2].copy_from_slice(&config.into_bytes());
        self.write_all_registers(registers).await
    }

    pub async fn set_channel_spacing(
        &mut self,
        channel_spacing: ChannelSpacing,
    ) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().await.unwrap();
        let idx = 2 * ReadRegIdx::SysConfig2 as usize;
        let mut config = SysConfig2::from_bytes([registers[idx], registers[idx + 1]]);

        config.set_space(channel_spacing);

        registers[idx..idx + 2].copy_from_slice(&config.into_bytes());
        self.write_all_registers(registers).await
    }

    pub async fn set_oscillator_enable(
        &mut self,
        enable: bool,
    ) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().await.unwrap();
        let idx = 2 * ReadRegIdx::Test1 as usize;
        let mut test1 = Test1::from_bytes([registers[idx], registers[idx + 1]]);

        test1.set_xoscen(enable);

        registers[idx..idx + 2].copy_from_slice(&test1.into_bytes());
        self.write_all_registers(registers).await
    }

    pub async fn get_chip_info(&mut self) -> Result<ChipInfo, Si470xError<I2C::Error>> {
        let registers: [u8; 32] = self.read_all_registers().await.unwrap();
        let idx = 2 * ReadRegIdx::ChipId as usize;
        let chip_id = ChipId::from_bytes([registers[idx], registers[idx + 1]]);
        Ok(ChipInfo {
            revision: chip_id.rev(),
            device: chip_id.dev(),
            firmware: chip_id.firmware(),
        })
    }

    pub async fn get_device_info(&mut self) -> Result<DeviceInfo, Si470xError<I2C::Error>> {
        let registers: [u8; 32] = self.read_all_registers().await.unwrap();
        let idx = 2 * ReadRegIdx::DeviceId as usize;
        let device_id = DeviceId::from_bytes([registers[idx], registers[idx + 1]]);
        Ok(DeviceInfo {
            pn: device_id.pn(),
            mfgid: device_id.mfgid(),
        })
    }

    pub async fn ping(&mut self) -> Result<(), Si470xError<I2C::Error>> {
        self.i2c
            .write(SI470X_I2C_ADDRESS, &[])
            .await
            .map_err(Si470xError::I2c)
    }
}
