#![no_std]

#[cfg(feature = "blocking")]
use embedded_hal::i2c::I2c as BlockingI2c;

#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c as AsyncI2c;

pub struct BlockingMode;
pub struct AsyncMode;

pub struct Si470x<I2C> {
    i2c: I2C,
}

const DEVICE_ADDRESS: u8 = 0x10;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Register {
    DeviceId = 0x0,
    ChipId = 0x1,
    PowerCfg = 0x2,
    Channel = 0x3,
    SysConfig1 = 0x4,
    SysConfig2 = 0x5,
    SysConfig3 = 0x6,
    Test1 = 0x7,
    Test2 = 0x8,
    BootConfig = 0x9,
    StatusRssi = 0xA,
    ReadChan = 0xB,
    RdsA = 0xC,
    RdsB = 0xD,
    RdsC = 0xE,
    RdsD = 0xF,
}

impl From<Register> for u8 {
    fn from(reg: Register) -> u8 {
        reg as u8
    }
}

#[derive(Debug)]
pub struct ChipInfo {
    pub revision: u8,
    pub device: u8,
    pub firmware: u8,
}

impl<I2C> Si470x<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn read_register(&mut self, reg: Register) -> Result<u16, I2C::Error>
    where
        I2C: BlockingI2c,
    {
        let reg_byte: u8 = reg.into();
        let mut buffer = [0u8; 2];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[reg_byte], &mut buffer)?;
        Ok(u16::from_be_bytes(buffer))
    }

    #[cfg(feature = "async")]
    pub async fn read_register_async(&mut self, reg: Register) -> Result<u16, I2C::Error>
    where
        I2C: AsyncI2c,
    {
        let reg_byte: u8 = reg.into();
        let mut buffer = [0u8; 2];
        self.i2c
            .write_read(DEVICE_ADDRESS, &[reg_byte], &mut buffer)
            .await?;
        Ok(u16::from_be_bytes(buffer))
    }
    pub fn write_register(&mut self, reg: Register, value: u8) -> Result<(), I2C::Error>
    where
        I2C: BlockingI2c,
    {
        let reg_byte: u8 = reg.into();
        let buf = [reg_byte, value];
        self.i2c.write(DEVICE_ADDRESS, &buf)?;

        Ok(())
    }

    #[cfg(feature = "async")]
    pub async fn write_register_async(&mut self, reg: Register, value: u8) -> Result<(), I2C::Error>
    where
        I2C: AsyncI2c,
    {
        let reg_byte: u8 = reg.into();
        let buf = [reg_byte, value];
        self.i2c.write(DEVICE_ADDRESS, &buf).await?;

        Ok(())
    }

    pub fn ping(&mut self) -> Result<(), I2C::Error>
    where
        I2C: BlockingI2c,
    {
        match self.i2c.write(DEVICE_ADDRESS, &[]) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    #[cfg(feature = "async")]
    pub async fn ping_async(&mut self) -> Result<(), I2C::Error>
    where
        I2C: AsyncI2c,
    {
        match self.i2c.write(DEVICE_ADDRESS, &[]).await {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn get_chip_info(&mut self) -> Result<ChipInfo, I2C::Error>
    where
        I2C: BlockingI2c,
    {
        let reg_val = self.read_register(Register::ChipId)?;
        let chip_info = ChipInfo {
            revision: ((reg_val & 0b1111110000000000) >> 10) as u8,
            device: ((reg_val & 0b0000001111000000) >> 6) as u8,
            firmware: (reg_val & 0b0000000000111111) as u8,
        };

        Ok(chip_info)
    }

    #[cfg(feature = "async")]
    pub async fn get_chip_info_async(&mut self) -> Result<ChipInfo, I2C::Error>
    where
        I2C: AsyncI2c,
    {
        let reg_val = self.read_register_async(Register::ChipId).await?;
        let chip_info = ChipInfo {
            revision: ((reg_val & 0b1111110000000000) >> 10) as u8,
            device: ((reg_val & 0b0000001111000000) >> 6) as u8,
            firmware: (reg_val & 0b0000000000111111) as u8,
        };

        Ok(chip_info)
    }
}
