#![no_std]

//#[cfg(feature = "blocking")]
// use embedded_hal::i2c::I2c as BlockingI2c;

// #[cfg(feature = "async")]
// use embedded_hal_async::i2c::I2c as AsyncI2c;

// Optional: marker types to distinguish modes at compile time
pub struct BlockingMode;
pub struct AsyncMode;

pub struct Si470x<I2C> {
    i2c: I2C,
}

impl<I2C> Si470x<I2C> {
    // Constructor
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }
}

// impl<I2C> Si470x<I2C, BlockingMode>
// where
//     I2C: BlockingI2c,
// {
//     pub fn read_register(&mut self, reg: u8) -> Result<u8, I2C::Error> {
//         let mut buf = [0u8; 1];
//         self.i2c.write_read(0x42, &[reg], &mut buf)?;
//         Ok(buf[0])
//     }
// }

// #[cfg(feature = "async")]
// impl<I2C> Si470x<I2C, AsyncMode>
// where
//     I2C: AsyncI2c,
// {
//     pub async fn read_register(&mut self, reg: u8) -> Result<u8, I2C::Error> {
//         let mut buf = [0u8; 1];
//         self.i2c.write_read(0x42, &[reg], &mut buf).await?;
//         Ok(buf[0])
//     }
// }
