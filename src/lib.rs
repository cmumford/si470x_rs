#![no_std]

// Declare the modules (this tells Rust to look for them)
pub mod driver_blocking;
pub mod driver_common;

#[cfg(feature = "async")]
pub mod driver_async;

// Re-export the public API so users can do `use si470x::Si470x`
pub use driver_blocking::Si470x as BlockingSi470x;
pub use driver_common::{ChipInfo, Si470xError};

#[cfg(feature = "async")]
pub use driver_async::Si470x as AsyncSi470x;

// Optional: default to blocking if no async feature
#[cfg(not(feature = "async"))]
pub type Si470x<I2C> = BlockingSi470x<I2C>;

#[cfg(feature = "async")]
pub type Si470x<I2C> = AsyncSi470x<I2C>;
