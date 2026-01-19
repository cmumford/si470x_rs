#![allow(dead_code)]

use modular_bitfield_msb::prelude::*;

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

#[derive(BitfieldSpecifier)]
#[bits = 1]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SeekMode {
    Wrap = 0b00, // wrap at upper/lower limit (default)
    Stop = 0b01, // stop at limit.
}

#[derive(BitfieldSpecifier)]
#[bits = 1]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SeekDirection {
    Down = 0b00, // Seek down (default)
    Up = 0b01,   // Seek up.
}

#[derive(BitfieldSpecifier)]
#[bits = 1]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SeekState {
    Disable = 0b00, // Disable seek (default)
    Enable = 0b01,  // Enable seek.
}

#[bitfield(bits = 16)]
pub struct PowerCfg {
    pub dsmute: bool,
    pub dmute: bool,
    pub mono: bool,
    #[skip]
    __: bool,
    pub rdsm: bool,
    pub skmode: SeekMode,
    pub seekup: SeekDirection,
    pub seek: SeekState,
    #[skip]
    __: bool,
    pub disable: bool,
    #[skip]
    __: B5,
    pub enable: bool,
}

#[bitfield(bits = 16)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Channel {
    pub tune: bool,
    #[skip]
    __: B5,
    pub chan: B10,
}

#[derive(BitfieldSpecifier)]
#[bits = 2]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Gpio3Mode {
    HighImpedance = 0b00, // High impedance (default).
    MonoStereo = 0b01,    // Mono/Stereo indicator (ST).
    Low = 0b10,
    High = 0b11,
}

// Setting STCIEN = 1 will generate a 5 ms low pulse on GPIO2 when the
// STC 0Ah[14] bit is set. Setting RDSIEN = 1 will generate a 5 ms low
// pulse on GPIO2 when the RDSR 0Ah[15] bit is set.
#[derive(BitfieldSpecifier)]
#[bits = 2]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Gpio2Mode {
    HighImpedance = 0b00,   // High impedance (default).
    StcRdsInterrupt = 0b01, // STC/RDS interrupt.
    Low = 0b10,
    High = 0b11,
}

#[derive(BitfieldSpecifier)]
#[bits = 2]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Gpio1Mode {
    HighImpedance = 0b00, // High impedance (default).
    __ = 0b01,            // Reserved.
    Low = 0b10,
    High = 0b11,
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
    pub gpio3: Gpio3Mode,
    pub gpio2: Gpio2Mode,
    pub gpio1: Gpio1Mode,
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
    __: B3,
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
    __: B14,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Test2 {
    #[skip]
    __: B16,
}

#[bitfield(bits = 16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BootConfig {
    #[skip]
    __: B16,
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

pub struct Registers {
    // Register bytes are ordered IAW ReadRegIdx.
    registers: [u8; 32],
    /// The last valid register in `registers`. This allows for partial reads
    /// from the device as registers 00h..09h are either fixed or read-only.
    /// Doing a partial read from 0Ah..0Fh is common to capture the dynamic
    /// status registers.
    last_valid_reg: ReadRegIdx,
}

impl Registers {
    #[inline]
    pub const fn new() -> Self {
        Self {
            registers: [0u8; 32],
            last_valid_reg: ReadRegIdx::BootConfig,
        }
    }

    #[inline]
    pub fn device_id(&self) -> DeviceId {
        DeviceId::from_bytes(self.get_raw(ReadRegIdx::DeviceId))
    }

    #[inline]
    pub fn chip_id(&self) -> ChipId {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::ChipId as u8);
        ChipId::from_bytes(self.get_raw(ReadRegIdx::ChipId))
    }

    #[inline]
    pub fn power_cfg(&self) -> PowerCfg {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::PowerCfg as u8);
        PowerCfg::from_bytes(self.get_raw(ReadRegIdx::PowerCfg))
    }

    #[inline]
    pub fn channel(&self) -> Channel {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::Channel as u8);
        Channel::from_bytes(self.get_raw(ReadRegIdx::Channel))
    }

    #[inline]
    pub fn sys_config1(&self) -> SysConfig1 {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::SysConfig1 as u8);
        SysConfig1::from_bytes(self.get_raw(ReadRegIdx::SysConfig1))
    }

    #[inline]
    pub fn sys_config2(&self) -> SysConfig2 {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::SysConfig2 as u8);
        SysConfig2::from_bytes(self.get_raw(ReadRegIdx::SysConfig2))
    }

    #[inline]
    pub fn sys_config3(&self) -> SysConfig3 {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::SysConfig3 as u8);
        SysConfig3::from_bytes(self.get_raw(ReadRegIdx::SysConfig3))
    }

    #[inline]
    pub fn test1(&self) -> Test1 {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::Test1 as u8);
        Test1::from_bytes(self.get_raw(ReadRegIdx::Test1))
    }

    #[inline]
    pub fn status_rssi(&self) -> StatusRssi {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::StatusRssi as u8);
        StatusRssi::from_bytes(self.get_raw(ReadRegIdx::StatusRssi))
    }

    #[inline]
    pub fn read_chan(&self) -> ReadChan {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::ReadChan as u8);
        ReadChan::from_bytes(self.get_raw(ReadRegIdx::ReadChan))
    }

    #[inline]
    pub fn rdsa_a(&self) -> RdsA {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::RdsA as u8);
        RdsA::from_bytes(self.get_raw(ReadRegIdx::RdsA))
    }

    #[inline]
    pub fn rdsa_b(&self) -> RdsB {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::RdsB as u8);
        RdsB::from_bytes(self.get_raw(ReadRegIdx::RdsB))
    }

    #[inline]
    pub fn rdsa_c(&self) -> RdsC {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::RdsC as u8);
        RdsC::from_bytes(self.get_raw(ReadRegIdx::RdsC))
    }

    #[inline]
    pub fn rdsa_d(&self) -> RdsD {
        assert!(self.last_valid_reg as u8 >= ReadRegIdx::RdsD as u8);
        RdsD::from_bytes(self.get_raw(ReadRegIdx::RdsD))
    }

    #[inline]
    pub fn set_power_cfg(&mut self, reg: PowerCfg) {
        self.set_raw(ReadRegIdx::PowerCfg, reg.into_bytes());
    }

    #[inline]
    pub fn set_channel(&mut self, reg: Channel) {
        self.set_raw(ReadRegIdx::Channel, reg.into_bytes());
    }

    #[inline]
    pub fn set_sys_config1(&mut self, reg: SysConfig1) {
        self.set_raw(ReadRegIdx::SysConfig1, reg.into_bytes());
    }

    #[inline]
    pub fn set_sys_config2(&mut self, reg: SysConfig2) {
        self.set_raw(ReadRegIdx::SysConfig2, reg.into_bytes());
    }

    #[inline]
    pub fn set_sys_config3(&mut self, reg: SysConfig3) {
        self.set_raw(ReadRegIdx::SysConfig3, reg.into_bytes());
    }

    #[inline]
    pub fn set_test1(&mut self, reg: Test1) {
        self.set_raw(ReadRegIdx::Test1, reg.into_bytes());
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.registers
    }

    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8; 32] {
        &mut self.registers
    }

    #[inline]
    fn get_raw(&self, idx: ReadRegIdx) -> [u8; 2] {
        let i = idx as usize * 2;
        [self.registers[i], self.registers[i + 1]]
    }

    #[inline]
    fn set_raw(&mut self, idx: ReadRegIdx, bytes: [u8; 2]) {
        let i = idx as usize * 2;
        self.registers[i] = bytes[0];
        self.registers[i + 1] = bytes[1];
    }
}
