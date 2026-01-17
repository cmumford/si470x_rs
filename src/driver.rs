#![allow(dead_code)]

use core::fmt;
use embedded_hal::{
    digital::{OutputPin, StatefulOutputPin},
    i2c::SevenBitAddress,
};
use maybe_async::maybe_async;

#[cfg(feature = "async")]
use embedded_hal_async::{delay::DelayNs, i2c::I2c};

#[cfg(feature = "sync")]
use embedded_hal::{delay::DelayNs, i2c::I2c};

// At the top level of lib.rs or a common module
#[cfg(all(feature = "sync", feature = "async"))]
compile_error!(
    "Features `sync` and `async` are mutually exclusive.\n\
     Choose exactly one:\n  \
     - `--features sync`   for blocking API\n  \
     - `--features async`  for async/await API"
);

// Prevent neither being enabled (require exactly one)
#[cfg(all(not(feature = "sync"), not(feature = "async")))]
compile_error!(
    "You must enable exactly one of the following features:\n  \
     - `sync`   → blocking / synchronous API\n  \
     - `async`  → async / await API (requires embedded-hal-async)"
);
use crate::registers::*;

pub const SI470X_I2C_ADDRESS: SevenBitAddress = 0x10;

#[derive(Debug)]
pub enum Si470xError<E> {
    I2c(E),
    TuneInProgress,
    OutOfRange,
    // ... add more if needed
}

impl<E: fmt::Debug> fmt::Display for Si470xError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I2c(e) => write!(f, "I²C error: {:?}", e),
            Self::TuneInProgress => write!(f, "Tune in progress"),
            Self::OutOfRange => write!(f, "Out of range"),
        }
    }
}

// Resets the Si470x radio chip into 2-wire (I²C) mode using what the datasheet
// calls busmode selection method 1.
//
// # Type parameters
// - `RST`: Reset pin (active low)
// - `SDA`: SDA pin
// - `SEN`: SEN pin
// - `D`: Delay provider
//
// All pins must implement `OutputPin`.
#[maybe_async]
pub async fn reset_radio_for_i2c<RST, SDA, SEN, D>(
    rst: &mut RST,
    sda: &mut SDA,
    sen: Option<&mut SEN>,
    delay: &mut D,
) -> Result<(), ()>
where
    RST: OutputPin + StatefulOutputPin,
    SDA: OutputPin + StatefulOutputPin,
    SEN: OutputPin + StatefulOutputPin,
    D: DelayNs,
{
    // Set initial state for 2-wire mode
    rst.set_low().map_err(|_| ())?;
    sda.set_low().map_err(|_| ())?;
    if let Some(sen_pin) = sen {
        sen_pin.set_high().map_err(|_| ())?;
    }

    // Hold for ≥5 ms
    delay.delay_ms(5).await;

    // Release reset
    rst.set_high().map_err(|_| ())?;

    // Wait for chip to stabilize (≥5 ms)
    delay.delay_ms(5).await;

    Ok(())
}

pub struct Si470x<I2C> {
    i2c: I2C,
}

#[maybe_async]
impl<I2C, E> Si470x<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    async fn read_all_registers(&mut self) -> Result<[u8; 32], Si470xError<I2C::Error>> {
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
    async fn write_all_registers(
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

    async fn modify_register<F>(
        &mut self,
        reg: ReadRegIdx,
        f: F,
    ) -> Result<(), Si470xError<I2C::Error>>
    where
        F: FnOnce([u8; 2]) -> Result<[u8; 2], Si470xError<I2C::Error>>,
    {
        let mut registers = self.read_all_registers().await?;
        let idx = 2 * reg as usize;
        let new_bytes = f([registers[idx], registers[idx + 1]])?;
        registers[idx..idx + 2].copy_from_slice(&new_bytes);
        self.write_all_registers(registers).await
    }

    // Enable or disable the device. Before disabling RDS should be disabled
    // according to the datasheet.
    pub async fn set_enable(&mut self, enable: bool) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::PowerCfg, |bytes| {
            let mut reg = PowerCfg::from_bytes(bytes);
            // Note: Datasheet says "The ENABLE bit should never be written to a 0".
            reg.set_enable(true);
            reg.set_disable(!enable);
            Ok(reg.into_bytes())
        })
        .await
    }

    pub async fn set_mute(&mut self, muted: bool) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::PowerCfg, |bytes| {
            let mut reg = PowerCfg::from_bytes(bytes);
            reg.set_dmute(muted);
            Ok(reg.into_bytes())
        })
        .await
    }

    // Set the radio volume. Volume is 4-bit unsigned.
    pub async fn set_volume(&mut self, volume: u8) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::SysConfig2, |bytes| {
            let mut reg = SysConfig2::from_bytes(bytes);
            reg.set_volume(volume);
            Ok(reg.into_bytes())
        })
        .await
    }

    pub async fn set_channel_spacing(
        &mut self,
        channel_spacing: ChannelSpacing,
    ) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::SysConfig2, |bytes| {
            let mut reg = SysConfig2::from_bytes(bytes);
            reg.set_space(channel_spacing);
            Ok(reg.into_bytes())
        })
        .await
    }

    // Set the RSSI seek threshold.
    pub async fn set_rssi_threshold(&mut self, seekth: u8) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::SysConfig2, |bytes| {
            let mut reg = SysConfig2::from_bytes(bytes);
            reg.set_seekth(seekth);
            Ok(reg.into_bytes())
        })
        .await
    }

    pub async fn set_channel(&mut self, channel: u16) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::Channel, |bytes| {
            let mut reg = Channel::from_bytes(bytes);
            if reg.tune() {
                return Err(Si470xError::TuneInProgress);
            }
            reg.set_chan(channel);
            Ok(reg.into_bytes())
        })
        .await
    }

    pub async fn set_oscillator_enable(
        &mut self,
        enable: bool,
    ) -> Result<(), Si470xError<I2C::Error>> {
        self.modify_register(ReadRegIdx::Test1, |bytes| {
            let mut reg = Test1::from_bytes(bytes);
            reg.set_xoscen(enable);
            Ok(reg.into_bytes())
        })
        .await
    }

    pub async fn get_chip_info(&mut self) -> Result<ChipInfo, Si470xError<I2C::Error>> {
        let registers: [u8; 32] = self.read_all_registers().await?;
        let idx = 2 * ReadRegIdx::ChipId as usize;
        let chip_id = ChipId::from_bytes([registers[idx], registers[idx + 1]]);
        Ok(ChipInfo {
            revision: chip_id.rev(),
            device: chip_id.dev(),
            firmware: chip_id.firmware(),
        })
    }

    pub async fn get_device_info(&mut self) -> Result<DeviceInfo, Si470xError<I2C::Error>> {
        let registers: [u8; 32] = self.read_all_registers().await?;
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
