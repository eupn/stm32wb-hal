//! STM32WB HAL implementation
//!
//! NOTE: This HAL implementation is under active development (as is the underlying
//! `embedded_hal` itself, together with its traits, some of which are unproven).

#![no_std]

pub use embedded_hal as hal;
/// TODO Eventually, this has to be fixed to be features picking
pub use stm32wb::stm32wb55 as pac;

#[cfg(feature = "rt")]
pub use self::pac::interrupt;

pub use crate::pac as device;
pub use crate::pac as stm32;

pub mod datetime;
pub mod delay;

pub mod dma;
pub mod dmamux;
pub mod flash;
pub mod gpio;
pub mod i2c;
pub mod ipcc;
pub mod lptim;
pub mod prelude;
pub mod pwm;
pub mod pwr;
pub mod rcc;
pub mod rtc;
pub mod smps;
pub mod time;
pub mod tl_mbox;
pub mod traits;
pub mod usb;
