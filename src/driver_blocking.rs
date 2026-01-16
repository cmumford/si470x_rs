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

    fn read_all_registers(&mut self) -> Result<[u8; 32], Si470xError<I2C::Error>> {
        let mut registers = [0u8; 32];

        // See command above ReadRegIdx for order of data.
        self.i2c
            .read(SI470X_I2C_ADDRESS, &mut registers)
            .map_err(Si470xError::I2c)?;
        Ok(registers)
    }

    // Write all "writable" registers (02h through 07h).
    // `registers` is in the read order defined by ReadRegIdx.
    fn write_all_registers(&mut self, registers: [u8; 32]) -> Result<(), Si470xError<I2C::Error>> {
        const START_IDX: usize = 2 * ReadRegIdx::PowerCfg as usize;
        const END_IDX: usize = 2 * ReadRegIdx::Test1 as usize;

        self.i2c
            .write(SI470X_I2C_ADDRESS, &registers[START_IDX..END_IDX + 2])
            .map_err(Si470xError::I2c)?;

        Ok(())
    }

    // Enable or disable the device. Before disabling RDS should be disabled
    // according to the datasheet.
    pub fn set_enable(&mut self, enable: bool) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().unwrap();
        let idx = 2 * ReadRegIdx::PowerCfg as usize;
        let mut config = PowerCfg::from_bytes([registers[idx], registers[idx + 1]]);

        // Note: Datasheet says "The ENABLE bit should never be written to a 0".
        config.set_enable(true);
        config.set_disable(!enable);

        registers[idx..idx + 2].copy_from_slice(&config.into_bytes());
        self.write_all_registers(registers)
    }

    pub fn set_mute(&mut self, muted: bool) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().unwrap();
        let idx = 2 * ReadRegIdx::PowerCfg as usize;
        let mut config = PowerCfg::from_bytes([registers[idx], registers[idx + 1]]);

        config.set_dmute(muted);

        registers[idx..idx + 2].copy_from_slice(&config.into_bytes());
        self.write_all_registers(registers)
    }

    // Set the radio volume. Volume is 4-bit unsigned.
    pub fn set_volume(&mut self, volume: u8) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().unwrap();
        let idx = 2 * ReadRegIdx::SysConfig2 as usize;
        let mut config = SysConfig2::from_bytes([registers[idx], registers[idx + 1]]);

        config.set_volume(volume);

        registers[idx..idx + 2].copy_from_slice(&config.into_bytes());
        self.write_all_registers(registers)
    }

    pub fn set_channel_spacing(
        &mut self,
        channel_spacing: ChannelSpacing,
    ) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().unwrap();
        let idx = 2 * ReadRegIdx::SysConfig2 as usize;
        let mut config = SysConfig2::from_bytes([registers[idx], registers[idx + 1]]);

        config.set_space(channel_spacing);

        registers[idx..idx + 2].copy_from_slice(&config.into_bytes());
        self.write_all_registers(registers)
    }

    pub fn set_channel(&mut self, channel: u16) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().unwrap();
        let idx = 2 * ReadRegIdx::Channel as usize;
        let mut reg = Channel::from_bytes([registers[idx], registers[idx + 1]]);

        if reg.tune() {
            return Err(Si470xError::TuneInProgress);
        }
        reg.set_chan(channel);

        registers[idx..idx + 2].copy_from_slice(&reg.into_bytes());
        self.write_all_registers(registers)
    }

    pub fn set_oscillator_enable(&mut self, enable: bool) -> Result<(), Si470xError<I2C::Error>> {
        let mut registers: [u8; 32] = self.read_all_registers().unwrap();
        let idx = 2 * ReadRegIdx::Test1 as usize;
        let mut test1 = Test1::from_bytes([registers[idx], registers[idx + 1]]);

        test1.set_xoscen(enable);

        registers[idx..idx + 2].copy_from_slice(&test1.into_bytes());
        self.write_all_registers(registers)
    }

    pub fn get_chip_info(&mut self) -> Result<ChipInfo, Si470xError<I2C::Error>> {
        let registers: [u8; 32] = self.read_all_registers().unwrap();
        let idx = 2 * ReadRegIdx::ChipId as usize;
        let chip_id = ChipId::from_bytes([registers[idx], registers[idx + 1]]);
        Ok(ChipInfo {
            revision: chip_id.rev(),
            device: chip_id.dev(),
            firmware: chip_id.firmware(),
        })
    }

    pub fn get_device_info(&mut self) -> Result<DeviceInfo, Si470xError<I2C::Error>> {
        let registers: [u8; 32] = self.read_all_registers().unwrap();
        let idx = 2 * ReadRegIdx::DeviceId as usize;
        let device_id = DeviceId::from_bytes([registers[idx], registers[idx + 1]]);
        Ok(DeviceInfo {
            pn: device_id.pn(),
            mfgid: device_id.mfgid(),
        })
    }

    pub fn ping(&mut self) -> Result<(), Si470xError<I2C::Error>> {
        self.i2c
            .write(SI470X_I2C_ADDRESS, &[])
            .map_err(Si470xError::I2c)
    }
}
