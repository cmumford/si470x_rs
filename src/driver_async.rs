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
        const START_IDX: usize = 2 * ReadRegIdx::PowerCfg as usize;
        const END_IDX: usize = 2 * ReadRegIdx::Test1 as usize;

        let write_slice = &registers[START_IDX..END_IDX];
        self.i2c
            .write(SI470X_I2C_ADDRESS, write_slice)
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

        let updated_bytes = config.into_bytes();
        registers[idx..idx + 2].copy_from_slice(&updated_bytes);
        self.write_all_registers(registers).await
    }

    pub async fn set_mute(&mut self, muted: bool) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().await.unwrap();
        let idx = 2 * ReadRegIdx::PowerCfg as usize;
        let mut config = PowerCfg::from_bytes([registers[idx], registers[idx + 1]]);

        config.set_dmute(muted);

        let updated_bytes = config.into_bytes();
        registers[idx..idx + 2].copy_from_slice(&updated_bytes);
        self.write_all_registers(registers).await
    }

    // Set the radio volume. Volume is 4-bit unsigned.
    pub async fn set_volume(&mut self, volume: u8) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().await.unwrap();
        let idx = 2 * ReadRegIdx::SysConfig2 as usize;
        let mut config = SysConfig2::from_bytes([registers[idx], registers[idx + 1]]);

        config.set_volume(volume);

        let updated_bytes = config.into_bytes();
        registers[idx..idx + 2].copy_from_slice(&updated_bytes);
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

        let updated_bytes = config.into_bytes();
        registers[idx..idx + 2].copy_from_slice(&updated_bytes);

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

        let updated_bytes = test1.into_bytes();
        registers[idx..idx + 2].copy_from_slice(&updated_bytes);

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

    pub async fn ping(&mut self) -> Result<(), Si470xError<I2C::Error>> {
        self.i2c
            .write(SI470X_I2C_ADDRESS, &[])
            .await
            .map_err(Si470xError::I2c)
    }
}
