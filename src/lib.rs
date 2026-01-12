#![no_std]

#[cfg(feature = "blocking")]
use embedded_hal::i2c::I2c as BlockingI2c;

#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c as AsyncI2c;

// Optional: marker types to distinguish modes at compile time
pub struct BlockingMode;
pub struct AsyncMode;

pub struct Si470x<I2C> {
    i2c: I2C,
}

const DEVICE_ADDRESS: u8 = 0x10;

impl<I2C> Si470x<I2C> {
    // Constructor
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    // Blocking version – always available when I2C implements BlockingI2c
    pub fn ping_blocking(&mut self) -> Result<(), I2C::Error>
    where
        I2C: BlockingI2c,
    {
        match self.i2c.write(DEVICE_ADDRESS, &[]) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    // Async version – only compiled when "async" feature is enabled
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
}
