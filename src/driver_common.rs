#![allow(dead_code)]

use core::fmt;
use embedded_hal::{
    delay::DelayNs,
    digital::{OutputPin, StatefulOutputPin},
    i2c::SevenBitAddress,
};
use modular_bitfield::prelude::*;

pub const SI470X_I2C_ADDRESS: SevenBitAddress = 0x10;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    DeviceId = 0x00,
    ChipId = 0x01,
    PowerCfg = 0x02,
    Channel = 0x03,
    SysConfig1 = 0x04,
    SysConfig2 = 0x05,
    SysConfig3 = 0x06,
    Test1 = 0x07,
    Test2 = 0x08,
    BootConfig = 0x09,
    StatusRssi = 0x0A,
    ReadChan = 0x0B,
    RdsA = 0x0C,
    RdsB = 0x0D,
    RdsC = 0x0E,
    RdsD = 0x0F,
}

impl From<Register> for u8 {
    fn from(reg: Register) -> u8 {
        reg as u8
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u16)]
pub enum PowerCfg {
    DSMUTE = 1 << 15,
    DMUTE = 1 << 14,
    MONO = 1 << 13,
    RDSM = 1 << 11,
    SKMODE = 1 << 10,
    SEEKUP = 1 << 9,
    SEEK = 1 << 8,
    DISABLE = 1 << 6,
    ENABLE = 1 << 0,
}

impl From<PowerCfg> for u16 {
    fn from(flag: PowerCfg) -> u16 {
        flag as u16
    }
}

#[derive(Specifier)]
#[bits = 2]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChannelSpacing {
    KHz200 = 0b00,
    KHz100 = 0b01,
    KHz50 = 0b10,
}

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SysConfig2 {
    pub seekth: B8,
    pub band: B2,
    #[bits = 2]
    pub space: ChannelSpacing,
    pub volume: B4,
}

#[bitfield]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Test1 {
    pub xoscen: bool,
    pub ahizen: bool,
    #[skip]
    unused: B14,
}

pub trait BitOps {
    fn set(self, flag: impl Into<u16>) -> Self;
    fn clear(self, flag: impl Into<u16>) -> Self;
    fn toggle(self, flag: impl Into<u16>) -> Self;
    fn contains(self, flag: impl Into<u16>) -> bool;
}

impl BitOps for u16 {
    #[inline(always)]
    fn set(self, flag: impl Into<u16>) -> u16 {
        self | flag.into()
    }
    #[inline(always)]
    fn clear(self, flag: impl Into<u16>) -> u16 {
        self & !flag.into()
    }
    #[inline(always)]
    fn toggle(self, flag: impl Into<u16>) -> u16 {
        self ^ flag.into()
    }
    #[inline(always)]
    fn contains(self, flag: impl Into<u16>) -> bool {
        (self & flag.into()) != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChipInfo {
    pub revision: u8,
    pub device: u8,
    pub firmware: u8,
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
