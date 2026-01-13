#![allow(dead_code)]

use core::fmt;

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
