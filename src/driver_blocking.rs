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
            .map_err(Si470xError::I2c)?;
        Ok(registers)
    }

    // Write all "writable" registers (02h through 07h).
    // `registers` is in the read order defined by ReadRegIdx.
    fn write_all_registers(&mut self, registers: [u8; 32]) -> Result<(), Si470xError<I2C::Error>> {
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
            .map_err(Si470xError::I2c)?;

        Ok(())
    }

    fn modify_register<F>(&mut self, reg: ReadRegIdx, f: F) -> Result<(), Si470xError<I2C::Error>>
    where
        F: FnOnce([u8; 2]) -> Result<[u8; 2], Si470xError<I2C::Error>>,
    {
        let mut registers = self.read_all_registers()?;
        let idx = 2 * reg as usize;
        let new_bytes = f([registers[idx], registers[idx + 1]])?;
        registers[idx..idx + 2].copy_from_slice(&new_bytes);
        self.write_all_registers(registers)
    }

    // Enable or disable the device. Before disabling RDS should be disabled
    // according to the datasheet.
    pub fn set_enable(&mut self, enable: bool) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::PowerCfg, |bytes| {
            let mut reg = PowerCfg::from_bytes(bytes);
            // Note: Datasheet says "The ENABLE bit should never be written to a 0".
            reg.set_enable(true);
            reg.set_disable(!enable);
            Ok(reg.into_bytes())
        })
    }

    pub fn set_mute(&mut self, muted: bool) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::PowerCfg, |bytes| {
            let mut reg = PowerCfg::from_bytes(bytes);
            reg.set_dmute(muted);
            Ok(reg.into_bytes())
        })
    }

    // Set the radio volume. Volume is 4-bit unsigned.
    pub fn set_volume(&mut self, volume: u8) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::SysConfig2, |bytes| {
            let mut reg = SysConfig2::from_bytes(bytes);
            reg.set_volume(volume);
            Ok(reg.into_bytes())
        })
    }

    pub fn set_channel_spacing(
        &mut self,
        channel_spacing: ChannelSpacing,
    ) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::SysConfig2, |bytes| {
            let mut reg = SysConfig2::from_bytes(bytes);
            reg.set_space(channel_spacing);
            Ok(reg.into_bytes())
        })
    }

    // Set the RSSI seek threshold.
    pub fn set_rssi_threshold(&mut self, seekth: u8) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::SysConfig2, |bytes| {
            let mut reg = SysConfig2::from_bytes(bytes);
            reg.set_seekth(seekth);
            Ok(reg.into_bytes())
        })
    }

    pub fn set_channel(&mut self, channel: u16) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::Channel, |bytes| {
            let mut reg = Channel::from_bytes(bytes);
            if reg.tune() {
                return Err(Si470xError::TuneInProgress);
            }
            reg.set_chan(channel);
            Ok(reg.into_bytes())
        })
    }

    pub fn set_oscillator_enable(&mut self, enable: bool) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::Test1, |bytes| {
            let mut reg = Test1::from_bytes(bytes);
            reg.set_xoscen(enable);
            Ok(reg.into_bytes())
        })
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
