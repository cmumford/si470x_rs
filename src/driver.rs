#![allow(dead_code)]

use maybe_async::maybe_async;

use core::fmt;
use embedded_hal::{
    digital::{OutputPin, StatefulOutputPin},
    i2c::SevenBitAddress,
};
use modular_bitfield_msb::prelude::*;

#[cfg(feature = "async")]
use embedded_hal_async::{delay::DelayNs, i2c::I2c};

#[cfg(not(feature = "async"))]
use embedded_hal::{delay::DelayNs, i2c::I2c};

// At the top level of lib.rs or a common module
#[cfg(all(feature = "sync", feature = "async"))]
compile_error!("Cannot enable both 'sync' and 'async' features at the same time");

pub const SI470X_I2C_ADDRESS: SevenBitAddress = 0x10;

// When reading from the Si470x, reading starts at register 0Ah (STATUSRSSI), and
// reads through to 0Fh (RDSD), and the wraps around to 00h through 09h.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ReadRegIdx {
    StatusRssi = 0x00,
    ReadChan = 0x01,
    RdsA = 0x02,
    RdsB = 0x03,
    RdsC = 0x04,
    RdsD = 0x05,
    DeviceId = 0x06,
    ChipId = 0x07,
    PowerCfg = 0x08,
    Channel = 0x09,
    SysConfig1 = 0x0A,
    SysConfig2 = 0x0B,
    SysConfig3 = 0x0C,
    Test1 = 0x0D,
    Test2 = 0x0E,
    BootConfig = 0x0F,
}

impl From<ReadRegIdx> for u8 {
    fn from(reg: ReadRegIdx) -> u8 {
        reg as u8
    }
}

#[bitfield(bits = 16)]
pub struct DeviceId {
    pub pn: B4,
    pub mfgid: B12,
}

#[bitfield(bits = 16)]
pub struct ChipId {
    pub rev: B6,
    pub dev: B4,
    pub firmware: B6,
}

#[bitfield(bits = 16)]
pub struct PowerCfg {
    pub dsmute: bool,
    pub dmute: bool,
    pub mono: bool,
    #[skip]
    unused: bool,
    pub rdsm: bool,
    pub skmode: bool,
    pub seekup: bool,
    pub seek: bool,
    #[skip]
    unused: bool,
    pub disable: bool,
    #[skip]
    unused: B5,
    pub enable: bool,
}

#[bitfield(bits = 16)]
pub struct Channel {
    pub tune: bool,
    #[skip]
    __: B5,
    pub chan: B10,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SysConfig1 {
    pub rdsien: bool,
    pub stcien: bool,
    #[skip]
    __: bool,
    pub rds: bool,
    pub de: bool,
    pub agcd: bool,
    #[skip]
    __: B2,
    pub blndadj: B2,
    pub gpio3: B2,
    pub gpio2: B2,
    pub gpio1: B2,
}

#[derive(BitfieldSpecifier)]
#[bits = 2]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChannelSpacing {
    KHz200 = 0b00,
    KHz100 = 0b01,
    KHz50 = 0b10,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SysConfig2 {
    pub seekth: B8,
    pub band: B2,
    #[bits = 2]
    pub space: ChannelSpacing,
    pub volume: B4,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SysConfig3 {
    pub smuter: B2,
    pub smutea: B2,
    #[skip]
    unused: B3,
    pub volext: bool,
    pub sksnr: B4,
    pub skcnt: B4,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Test1 {
    pub xoscen: bool,
    pub ahizen: bool,
    #[skip]
    unused: B14,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Test2 {
    #[skip]
    unused: B16,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BootConfig {
    #[skip]
    unused: B16,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StatusRssi {
    pub rdsrr: bool,
    pub stc: bool,
    pub sf_bl: bool,
    pub afcrl: bool,
    pub rdss: bool,
    pub blera: B2,
    pub st: bool,
    pub rssi: B8,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ReadChan {
    pub blerb: B2,
    pub blerc: B2,
    pub blerd: B2,
    pub readchan: B10,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RdsA {
    pub rdsa: B16,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RdsB {
    pub rdsb: B16,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RdsC {
    pub rdsc: B16,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RdsD {
    pub rdsd: B16,
}

#[derive(Debug, Clone, Copy)]
pub struct ChipInfo {
    pub revision: u8,
    pub device: u8,
    pub firmware: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceInfo {
    pub pn: u8,
    pub mfgid: u16,
}

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
