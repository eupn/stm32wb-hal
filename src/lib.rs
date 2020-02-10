//! STM32WB HAL implementation
//!
//! NOTE: This HAL implementation is under active development (as is the underlying
//! `embedded_hal` itself, together with its traits, some of which are unproven).

#![no_std]

pub use embedded_hal as hal;
pub use stm32wb_pac as pac;

#[cfg(feature = "rt")]
pub use self::pac::interrupt;

pub use crate::pac as device;
pub use crate::pac as stm32;

pub mod datetime;
pub mod delay;

pub mod flash;
pub mod gpio;
pub mod i2c;
pub mod prelude;
pub mod rcc;
pub mod time;
pub mod usb;
