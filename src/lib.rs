#![no_std]

mod driver;
mod registers;

/// Minimum delay after enabling the crystal oscillator (via setting the XOSCEN
/// bit) to allow stabilization.
///
/// AN230 says:
/// > Provide a sufficient delay before setting the ENABLE bit to ensure that
/// > the oscillator has stabilized. The delay will vary depending on the
/// > external oscillator circuit and the ESR of the crystal, and it should
/// > include margin to allow for device tolerances. The recommended minimum
/// > delay is no less than 500 ms. A similar delay may be necessary for some
/// > external oscillator circuits. Determine the necessary stabilization time
/// > for the clock source in the system.
pub const OSCILLATOR_ENABLE_MIN_DELAY_MS: u32 = 500;

/// Time required after power-up before the device is ready for commands.
/// See datasheet table 8.
///
/// From datasheet:
/// > Do not enable STC interrupts before the powerup time is complete. If STC
/// > interrupts are enabled before the powerup time is complete, an interrupt
/// > will be generated within the powerup interval when the initial default
/// > tune operation is complete. See "AN230: Si4700/01/02/03 Programmer’s
/// > Guide" for more information.
pub const POWERUP_TIME_MS: u32 = 110;

pub use driver::{Si470x, Si470xError, reset_radio_for_i2c};
pub use registers::{
    ChipId, DeviceId, Gpio2Mode, RdsErrorCnt, ReadRegIdx, Registers, SeekDirection, SeekMode,
    SeekState,
};
