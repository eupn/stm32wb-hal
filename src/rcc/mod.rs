//! Reset and Clock Control

use core::cmp;

mod config;

pub use config::*;

use crate::stm32::{rcc, RCC};
use cast::u32;

use crate::flash::ACR;
use crate::time::{Hertz, U32Ext};

/// HSI speed
pub const HSI_FREQ: u32 = 16_000_000;

pub struct Rcc {
    pub clocks: Clocks,
    pub(crate) rb: RCC,
}

impl Rcc {
    pub fn freeze(self, _config: config::Config) -> Self {
        self
    }
}

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            clocks: Clocks::default(),
            rb: self,
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy, Debug)]
pub struct Clocks {
    sysclk: Hertz,

    hclk1: Hertz,
    hclk2: Hertz,
    hclk4: Hertz,

    systick: Hertz,

    pclk1: Hertz,
    tim_pclk1: Hertz,

    pclk2: Hertz,
    tim_pclk2: Hertz,

    lsi: Hertz,

    rtcclk: Hertz,

    // Clocked by SAI, disabled by default
    rng: Option<Hertz>,
    adc: Option<Hertz>,
    clk48: Option<Hertz>,
    sai1: Option<Hertz>,

    i2c1: Hertz,
    i2c3: Hertz,

    usart1: Hertz,
    lpuart1: Hertz,

    lptim1: Hertz,
    lptim2: Hertz,
}

impl Default for Clocks {
    /// Default clock frequencies right after power-on reset.
    fn default() -> Self {
        Clocks {
            sysclk: 4.mhz(),
            hclk1: 4.mhz(),
            hclk2: 4.mhz(),
            hclk4: 4.mhz(),
            systick: 4.mhz(),
            pclk1: 4.mhz(),
            tim_pclk1: 4.mhz(),
            pclk2: 4.mhz(),
            tim_pclk2: 4.mhz(),
            lsi: 32.khz(),
            rtcclk: 32.khz(),
            rng: None,
            adc: None,
            clk48: None,
            sai1: None,
            i2c1: 4.mhz(),
            i2c3: 4.mhz(),
            usart1: 4.mhz(),
            lpuart1: 4.mhz(),
            lptim1: 4.mhz(),
            lptim2: 4.mhz(),
        }
    }
}

impl Clocks {
    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
