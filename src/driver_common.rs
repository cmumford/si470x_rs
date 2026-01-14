#![allow(dead_code)]

use core::fmt;
use embedded_hal::{
    delay::DelayNs,
    digital::{OutputPin, StatefulOutputPin},
    i2c::SevenBitAddress,
};
use modular_bitfield_msb::prelude::*;

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
    InvalidResponse,
    // ... add more if needed
}

impl<E: fmt::Debug> fmt::Display for Si470xError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I2c(e) => write!(f, "I²C error: {:?}", e),
            Self::InvalidResponse => write!(f, "Invalid response from device"),
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
pub fn reset_radio_for_i2c<RST, SDA, SEN, D>(
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
    delay.delay_ms(5);

    // Release reset
    rst.set_high().map_err(|_| ())?;

    // Wait for chip to stabilize (≥5 ms)
    delay.delay_ms(5);

    Ok(())
}
