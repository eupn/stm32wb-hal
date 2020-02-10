//! Reset and Clock Control

mod config;

pub use config::*;

use crate::stm32::RCC;

use crate::flash::ACR;
use crate::time::{Hertz, U32Ext};

use cortex_m_semihosting::hprintln;

/// HSI frequency.
pub const HSI_FREQ: u32 = 16_000_000;

/// On WB55 HSE frequency is fixed with 32 MHz.
pub const HSE_FREQ: u32 = 32_000_000;

pub struct Rcc {
    pub clocks: Clocks,
    pub(crate) rb: RCC,
}

impl Rcc {
    pub fn freeze(mut self, config: config::Config, acr: &mut ACR) -> Self {
        // Select system clock source
        match &config.sysclk_src {
            SysClkSrc::Msi(msi_range) => {}
            SysClkSrc::Hsi => {}
            SysClkSrc::HseSys(hse_div) => {}
            SysClkSrc::Pll(src) => {
                self.configure_and_wait_for_pll(&config.pll_cfg, src);
                if let Some(pllclk) = self.clocks.pllclk {
                    self.clocks.sysclk = pllclk;
                }

                // Configure CPU1 and CPU2 dividers
                self.rb
                    .cfgr
                    .modify(|_r, w| unsafe { w.hpre().bits(config.cpu1_hdiv as u8) });
                self.rb
                    .extcfgr
                    .modify(|_r, w| unsafe { w.c2hpre().bits(config.cpu2_hdiv as u8) });

                // Configure FLASH wait states
                acr.acr().write(|w| unsafe {
                    w.latency().bits(if self.clocks.sysclk.0 <= 18_000_000 {
                        0
                    } else if self.clocks.sysclk.0 <= 36_000_000 {
                        1
                    } else if self.clocks.sysclk.0 <= 54_000_000 {
                        2
                    } else {
                        3
                    })
                });

                // Configure SYSCLK mux to use PLL clock
                self.rb.cfgr.modify(|_r, w| unsafe { w.sw().bits(0b11) });

                // Wait for SYSCLK to switch
                while self.rb.cfgr.read().sw() != 0b11 {}
            }
        }

        self
    }

    fn configure_and_wait_for_pll(&mut self, config: &PllConfig, src: &PllSrcMux) {
        // Select PLL and PLLSAI1 clock source [RM0434, p. 233]
        let (f_input, src_bits) = match src {
            PllSrcMux::Msi(_range) => {
                todo!();

                let f_input = 0;
                (f_input, 0b01)
            }
            PllSrcMux::Hsi => (HSI_FREQ, 0b10),
            PllSrcMux::Hse(div) => {
                let (divided, f_input) = match div {
                    HseDivider::NotDivided => (false, HSE_FREQ),
                    HseDivider::Div2 => (true, HSE_FREQ / 2),
                };

                // Configure HSE divider and enable it
                self.rb
                    .cr
                    .modify(|_, w| w.hsepre().bit(divided).hseon().set_bit());
                // Wait for HSE startup
                while !self.rb.cr.read().hserdy().bit_is_set() {}

                (f_input, 0b11)
            }
        };

        let pllp = config.p.map(|p| {
            assert!(p > 1);
            assert!(p <= 32);
            (p - 1) & 0b11111
        });

        let pllq = config.q.map(|q| {
            assert!(q > 1);
            assert!(q <= 8);
            (q - 1) & 0b111
        });

        // Set R value
        assert!(config.r > 1);
        assert!(config.r <= 8);
        let pllr = (config.r - 1) & 0b111;

        // Set N value
        assert!(config.n > 7);
        assert!(config.n <= 86);
        let plln = config.n & 0b1111111;

        // Set M value
        assert!(config.m > 0);
        assert!(config.m <= 8);
        let pllm = (config.m - 1) & 0b111;

        let vco = f_input / config.m as u32 * config.n as u32;
        let f_pllr = vco / config.r as u32;

        assert!(f_pllr <= 64_000_000);

        self.clocks.pllclk = Some(f_pllr.hz());

        if let Some(pllp) = pllp {
            let f_pllp = vco / (pllp + 1) as u32;
            assert!(f_pllp <= 64_000_000);

            self.clocks.pllp = Some(f_pllp.hz());
        }

        if let Some(pllq) = pllq {
            let f_pllq = vco / (pllq + 1) as u32;
            assert!(f_pllq <= 64_000_000);

            self.clocks.pllq = Some(f_pllq.hz());
        }

        // Set PLL coefficients
        #[rustfmt::skip]
        {
            self.rb.pllcfgr.modify(|_, w| unsafe {
                w.pllsrc().bits(src_bits)
                    .pllm().bits(pllm)
                    .plln().bits(plln)
                    .pllr().bits(pllr).pllren().set_bit()
                    .pllp().bits(pllp.unwrap_or(1)).pllpen().bit(pllp.is_some())
                    .pllq().bits(pllq.unwrap_or(1)).pllqen().bit(pllq.is_some())
            });
        }

        // Enable PLL and wait for setup
        self.rb.cr.modify(|_, w| w.pllon().set_bit());
        while !self.rb.cr.read().pllrdy().bit_is_set() {}
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

    pllclk: Option<Hertz>,
    pllq: Option<Hertz>,
    pllp: Option<Hertz>,
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
            pllclk: None,
            pllq: None,
            pllp: None,
        }
    }
}

impl Clocks {
    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }

    pub fn pclk1(&self) -> Hertz {
        self.pclk1
    }
}
