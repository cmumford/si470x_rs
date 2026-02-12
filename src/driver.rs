#![allow(dead_code)]
#![allow(async_fn_in_trait)]

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
#[maybe_async(AFIT)]
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

/// Core trait for interacting with a Si470x-family FM tuner via I²C.
///
/// This trait abstracts the low-level register read/write operations and
/// is implemented by `Si470x<I2C>` (and can be implemented by mocks for testing).
#[maybe_async(AFIT)]
pub trait Tuner<E> {
    /// Reads a contiguous block of registers starting from register 0Ah
    /// up to and including the specified register.
    ///
    /// The returned `Registers` struct contains the requested range of registers.
    async fn read_registers(
        &mut self,
        up_to_and_including_reg: ReadRegIdx,
    ) -> Result<Registers, Si470xError<E>>;

    /// Reads all readable registers (from 0Ah → 0Fh, then wraps to 00h → 01h, etc.).
    async fn read_all_registers(&mut self) -> Result<Registers, Si470xError<E>>;

    /// Writes the modified registers back to the device.
    ///
    /// Only registers that are writable (starting from 02h) are sent.
    /// The implementation must respect the Si470x write sequence (starting at 02h).
    async fn write_registers(&mut self, registers: &Registers) -> Result<(), Si470xError<E>>;

    /// Simple I²C ping / presence check (write zero bytes).
    async fn ping(&mut self) -> Result<(), Si470xError<E>>;
}

pub struct Si470x<I2C>
where
    I2C: I2c,
{
    i2c: I2C,
}

#[maybe_async(AFIT)]
impl<I2C, E> Si470x<I2C>
where
    I2C: I2c<Error = E> + I2c,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    // Enable or disable the device. Before disabling RDS should be disabled
    // according to the datasheet.
    pub async fn set_enable(&mut self, enable: bool) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::PowerCfg).await?;
        let mut reg = registers.power_cfg();
        // Note: Datasheet says "The ENABLE bit should never be written to a 0".
        reg.set_enable(true);
        reg.set_disable(!enable);
        registers.set_power_cfg(reg);
        self.write_registers(&registers).await
    }

    pub async fn set_softmute(&mut self, muted: bool) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::PowerCfg).await?;
        let mut reg = registers.power_cfg();
        let mute_disabled = !muted;
        reg.set_dsmute(mute_disabled);
        registers.set_power_cfg(reg);
        self.write_registers(&registers).await
    }

    pub async fn set_mute(&mut self, muted: bool) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::PowerCfg).await?;
        let mut reg = registers.power_cfg();
        let mute_disabled = !muted;
        reg.set_dmute(mute_disabled);
        registers.set_power_cfg(reg);
        self.write_registers(&registers).await
    }

    pub async fn set_mono(&mut self, mono: bool) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::PowerCfg).await?;
        let mut reg = registers.power_cfg();
        reg.set_mono(mono);
        registers.set_power_cfg(reg);
        self.write_registers(&registers).await
    }

    pub async fn set_rds_mode(&mut self, mode: RdsMode) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::PowerCfg).await?;
        let mut reg = registers.power_cfg();
        reg.set_rdsm(mode);
        registers.set_power_cfg(reg);
        self.write_registers(&registers).await
    }

    pub async fn set_seek(
        &mut self,
        mode: SeekMode,
        direction: SeekDirection,
        state: SeekState,
    ) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::PowerCfg).await?;
        let mut reg = registers.power_cfg();
        reg.set_skmode(mode);
        reg.set_seekup(direction);
        reg.set_seek(state);
        registers.set_power_cfg(reg);
        self.write_registers(&registers).await
    }

    pub async fn clear_tune_seek_bits(&mut self) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_all_registers().await?;
        {
            let mut reg = registers.channel();
            reg.set_tune(false);
            registers.set_channel(reg);
        }
        {
            let mut reg = registers.power_cfg();
            reg.set_seek(SeekState::Disable);
            registers.set_power_cfg(reg);
        }
        self.write_registers(&registers).await
    }

    pub async fn set_channel(&mut self, channel: u16) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_all_registers().await?;
        let mut creg = registers.channel();
        if creg.tune() {
            return Err(Si470xError::TuneInProgress);
        }
        let mut preg = registers.power_cfg();
        if preg.seek() == SeekState::Enable {
            preg.set_seek(SeekState::Disable);
            registers.set_power_cfg(preg);
        }
        creg.set_chan(channel);
        creg.set_tune(true);
        registers.set_channel(creg);
        self.write_registers(&registers).await
    }

    // Set the radio volume. Volume is 4-bit unsigned.
    pub async fn set_volume(&mut self, volume: u8) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::SysConfig2).await?;
        let mut reg = registers.sys_config2();
        reg.set_volume(volume);
        registers.set_sys_config2(reg);
        self.write_registers(&registers).await
    }

    pub async fn set_channel_spacing(
        &mut self,
        channel_spacing: ChannelSpacing,
    ) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::SysConfig2).await?;
        let mut reg = registers.sys_config2();
        reg.set_space(channel_spacing);
        registers.set_sys_config2(reg);
        self.write_registers(&registers).await
    }

    // Set the RSSI seek threshold.
    pub async fn set_rssi_threshold(&mut self, seekth: u8) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::SysConfig2).await?;
        let mut reg = registers.sys_config2();
        reg.set_seekth(seekth);
        registers.set_sys_config2(reg);
        self.write_registers(&registers).await
    }

    pub async fn set_oscillator_enable(&mut self, enable: bool) -> Result<(), Si470xError<E>> {
        let mut registers = self.read_registers(ReadRegIdx::Test1).await?;
        let mut reg = registers.test1();
        reg.set_xoscen(enable);
        registers.set_test1(reg);
        self.write_registers(&registers).await
    }
}

#[maybe_async(AFIT)]
impl<I2C, E> Tuner<E> for Si470x<I2C>
where
    I2C: I2c<Error = E>,
{
    async fn read_registers(
        &mut self,
        up_to_and_including_reg: ReadRegIdx,
    ) -> Result<Registers, Si470xError<E>> {
        let mut registers = Registers::new(up_to_and_including_reg);

        let num_registers: usize = up_to_and_including_reg as usize + 1;
        let num_bytes: usize = 2 * num_registers;

        // From the datasheet:
        // "For read operations, the device acknowledge is followed by an eight
        // bit data word shifted out on falling SCLK edges. An internal address
        // counter automatically increments to allow continuous data byte
        // reads, starting with the upper byte of register 0Ah, followed by the
        // lower byte of register 0Ah, and onward until the lower byte of the
        // last register is reached. The internal address counter then
        // automatically wraps around to the upperbyte of register 00h and
        // proceeds from there until continuous reads cease."

        // See command above ReadRegIdx for order of data.
        #[cfg(feature = "sync")]
        let result = self
            .i2c
            .read(SI470X_I2C_ADDRESS, registers.as_bytes_mut_n(num_bytes));
        #[cfg(feature = "async")]
        let result = self
            .i2c
            .read(SI470X_I2C_ADDRESS, registers.as_bytes_mut_n(num_bytes))
            .await;

        result.map_err(Si470xError::I2c)?;
        Ok(registers)
    }

    async fn read_all_registers(&mut self) -> Result<Registers, Si470xError<E>> {
        #[cfg(feature = "sync")]
        let result = self.read_registers(ReadRegIdx::BootConfig);
        #[cfg(feature = "async")]
        let result = self.read_registers(ReadRegIdx::BootConfig).await;
        result
    }

    async fn write_registers(&mut self, registers: &Registers) -> Result<(), Si470xError<E>> {
        if (registers.get_last_valid_reg() as u8) < (ReadRegIdx::PowerCfg as u8) {
            // This is only a partial set of registers and does not include any
            // registers that can be written.
            return Err(Si470xError::OutOfRange);
        }
        // From the datasheet:
        // "An internal address counter automatically increments to allow
        // continuous data byte writes, starting with the upper byte of register
        // 02h, followed by the lower byte of register 02h, and onward until
        // the lower byte of the last register is reached. The internal address
        // counter then automatically wraps around to the upper byte of
        // register 00h and proceeds from there until continuous writes end."
        const START_IDX: usize = 2 * ReadRegIdx::PowerCfg as usize;
        let end_idx: usize = 2 * registers.get_last_valid_reg() as usize;

        #[cfg(feature = "sync")]
        let result = self.i2c.write(
            SI470X_I2C_ADDRESS,
            &registers.as_bytes()[START_IDX..end_idx + 2],
        );
        #[cfg(feature = "async")]
        let result = self
            .i2c
            .write(
                SI470X_I2C_ADDRESS,
                &registers.as_bytes()[START_IDX..end_idx + 2],
            )
            .await;

        result.map_err(Si470xError::I2c)?;

        Ok(())
    }

    async fn ping(&mut self) -> Result<(), Si470xError<E>> {
        self.i2c
            .write(SI470X_I2C_ADDRESS, &[])
            .await
            .map_err(Si470xError::I2c)
    }
}
