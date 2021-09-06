//! Reset and Clock Control

mod config;
mod mux;

pub use config::*;
pub use mux::*;

use crate::stm32::RCC;

use crate::flash::ACR;
use crate::time::{Hertz, U32Ext};

/// HSI frequency.
pub const HSI_FREQ: u32 = 16_000_000;

/// On WB55 HSE frequency is fixed with 32 MHz.
pub const HSE_FREQ: u32 = 32_000_000;

pub struct Rcc {
    pub clocks: Clocks,
    pub config: config::Config,
    pub rb: RCC,
}

impl Rcc {
    pub fn apply_clock_config(mut self, config: config::Config, acr: &mut ACR) -> Self {
        self.config = config.clone();

        // Enable backup domain access to access LSE/RTC registers
        crate::pwr::set_backup_access(true);

        // Configure LSE if needed
        if config.lse {
            self.rb.bdcr.modify(|_, w| w.lseon().set_bit());
            while !self.rb.bdcr.read().lserdy().bit_is_set() {}

            self.clocks.lse = Some(32768.hz());
        }

        // Configure LSI1 if needed
        if config.lsi1 {
            self.rb.csr.modify(|_, w| w.lsi1on().clear_bit());
        } else {
            self.rb.csr.modify(|_, w| w.lsi1on().set_bit());
            while !self.rb.csr.read().lsi1rdy().bit_is_set() {}
        }

        // Select system clock source
        let sysclk_bits = match &config.sysclk_src {
            SysClkSrc::Msi(_msi_range) => todo!(),
            SysClkSrc::Hsi => todo!(),
            SysClkSrc::HseSys(hse_div) => {
                // Actually turn on and use HSE....
                let (divided, f_input) = match hse_div {
                    HseDivider::NotDivided => (false, HSE_FREQ),
                    HseDivider::Div2 => (true, HSE_FREQ / 2),
                };
                self.rb.cr.modify(|_, w| w.hsepre().bit(divided).hseon().set_bit());
                // Wait for HSE startup
                while !self.rb.cr.read().hserdy().bit_is_set() {}

                self.clocks.hse = Some(HSE_FREQ.hz());
                self.clocks.sysclk = f_input.hz();

                0b10
            }
            SysClkSrc::Pll(src) => {
                self.configure_and_wait_for_pll(&config.pll_cfg, src);
                if let Some(pllclk) = self.clocks.pllclk {
                    self.clocks.sysclk = pllclk;
                }

                0b11
            }
        };

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

        // Configure SYSCLK mux to use selected clock
        self.rb
            .cfgr
            .modify(|_r, w| unsafe { w.sw().bits(sysclk_bits) });

        // Wait for SYSCLK to switch
        while self.rb.cfgr.read().sw() != sysclk_bits {}

        // Configure CPU1 and CPU2 dividers
        self.clocks.hclk1 = (self.clocks.sysclk.0 / config.cpu1_hdiv.divisor()).hz();
        self.clocks.hclk2 = (self.clocks.sysclk.0 / config.cpu2_hdiv.divisor()).hz();
        self.clocks.hclk4 = (self.clocks.sysclk.0 / config.hclk_hdiv.divisor()).hz();

        self.rb
            .cfgr
            .modify(|_r, w| unsafe { w.hpre().bits(config.cpu1_hdiv as u8) });
        self.rb.extcfgr.modify(|_r, w| unsafe {
            w.c2hpre()
                .bits(config.cpu2_hdiv as u8)
                .shdhpre()
                .bits(config.hclk_hdiv as u8)
        });

        // Wait for prescaler values to apply
        while !self.rb.cfgr.read().hpref().bit_is_set() {}
        while !self.rb.extcfgr.read().shdhpref().bit_is_set() {}

        // Apply PCLK1(APB1) / PCLK2(APB2) values
        self.rb.cfgr.modify(|_r, w| unsafe {
            w.ppre1()
                .bits(config.apb1_div as u8)
                .ppre2()
                .bits(config.apb2_div as u8)
        });

        while !self.rb.cfgr.read().ppre1f().bit_is_set() {}
        while !self.rb.cfgr.read().ppre2f().bit_is_set() {}

        self.clocks.pclk1 = (self.clocks.hclk1.0 / config.apb1_div.divisor()).hz();
        self.clocks.pclk2 = (self.clocks.hclk1.0 / config.apb2_div.divisor()).hz();

        // Select USB clock source
        if let Some(usb_src) = config.usb_src {
            self.rb
                .ccipr
                .modify(|_r, w| unsafe { w.clk48sel().bits(usb_src as u8) });

            self.clocks.clk48 = match usb_src {
                UsbClkSrc::Hsi48 => todo!(),
                UsbClkSrc::PllSai1Q => todo!(),
                UsbClkSrc::PllQ => self.clocks.pllq,
                UsbClkSrc::Msi => todo!(),
            };
        }

        // Set RF wake-up clock source
        self.rb
            .csr
            .modify(|_, w| unsafe { w.rfwkpsel().bits(config.rf_wkp_src as u8) });

        // Set LPTIM1 & LPTIM2 clock source
        self.rb
            .ccipr
            .modify(|_, w| unsafe { w.lptim1sel().bits(config.lptim1_src as u8) });
        self.rb
            .ccipr
            .modify(|_, w| unsafe { w.lptim2sel().bits(config.lptim2_src as u8) });

        match config.lptim1_src {
            LptimClkSrc::Pclk => self.clocks.lptim1 = self.clocks.pclk1(),
            LptimClkSrc::Lsi => self.clocks.lptim1 = self.clocks.lsi(),
            LptimClkSrc::Hsi16 => self.clocks.lptim1 = self.clocks.hsi16(),
            LptimClkSrc::Lse => self.clocks.lptim1 = self.clocks.lse().unwrap(),
        }

        match config.lptim2_src {
            LptimClkSrc::Pclk => self.clocks.lptim2 = self.clocks.pclk1(),
            LptimClkSrc::Lsi => self.clocks.lptim2 = self.clocks.lsi(),
            LptimClkSrc::Hsi16 => self.clocks.lptim2 = self.clocks.hsi16(),
            LptimClkSrc::Lse => self.clocks.lptim2 = self.clocks.lse().unwrap(),
        }

        self
    }

    #[allow(unreachable_code)] // TODO: remove
    fn configure_and_wait_for_pll(&mut self, config: &PllConfig, src: &PllSrc) {
        // Select PLL and PLLSAI1 clock source [RM0434, p. 233]
        let (f_input, src_bits) = match src {
            PllSrc::Msi(_range) => {
                todo!();

                let f_input = 0;
                (f_input, 0b01)
            }
            PllSrc::Hsi => (HSI_FREQ, 0b10),
            PllSrc::Hse(div) => {
                self.clocks.hse = Some(HSE_FREQ.hz());

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
        self.rb.pllcfgr.modify(|_, w| unsafe {
            w.pllsrc()
                .bits(src_bits)
                .pllm()
                .bits(pllm)
                .plln()
                .bits(plln)
                .pllr()
                .bits(pllr)
                .pllren()
                .set_bit()
                .pllp()
                .bits(pllp.unwrap_or(1))
                .pllpen()
                .bit(pllp.is_some())
                .pllq()
                .bits(pllq.unwrap_or(1))
                .pllqen()
                .bit(pllq.is_some())
        });

        // Enable PLL and wait for setup
        self.rb.cr.modify(|_, w| w.pllon().set_bit());
        while !self.rb.cr.read().pllrdy().bit_is_set() {}
    }

    /// Enables or disables IPCC peripheral clock.
    pub fn set_ipcc(&mut self, enabled: bool) {
        self.rb.ahb3enr.modify(|_, w| w.ipccen().bit(enabled));

        // Single memory access delay after peripheral is enabled.
        // This dummy read uses `read_volatile` internally, so it shouldn't be removed by an optimizer.
        let _ = self.rb.ahb3enr.read().ipccen();
    }

    /// Sets default clock source after exit from STOP modes.
    pub fn set_stop_wakeup_clock(&mut self, stop_wakeup_clock: StopWakeupClock) {
        let bit = match stop_wakeup_clock {
            StopWakeupClock::MSI => false,
            StopWakeupClock::HSI16 => true,
        };

        self.rb.cfgr.modify(|_, w| w.stopwuck().bit(bit));
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
            config: Config::default(),
            rb: self,
        }
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy, Debug)]
pub struct Clocks {
    sysclk: Hertz, // Max 64 MHz

    hclk1: Hertz, // Max 64 MHz
    hclk2: Hertz, // Max 32 MHz
    hclk4: Hertz, // Max 64 MHz

    systick: Hertz, // Max 64 MHz

    pub(crate) lse: Option<Hertz>,
    pub(crate) hse: Option<Hertz>, // Must be exactly 32 MHz

    pclk1: Hertz,
    tim_pclk1: Hertz,

    pclk2: Hertz,
    tim_pclk2: Hertz,

    pub(crate) lsi: Hertz,

    pub(crate) rtcclk: Hertz,

    rng: Option<Hertz>,
    adc: Option<Hertz>,
    clk48: Option<Hertz>,
    sai1: Option<Hertz>,

    i2c1: Hertz,
    i2c3: Hertz,

    usart1: Hertz,
    lpuart1: Hertz,

    pub(crate) lptim1: Hertz,
    pub(crate) lptim2: Hertz,

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
            lse: None,
            hse: None,
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

    pub fn pclk2(&self) -> Hertz {
        self.pclk2
    }

    pub fn lsi(&self) -> Hertz {
        self.lsi
    }

    pub fn lse(&self) -> Option<Hertz> {
        self.lse
    }

    pub fn hsi16(&self) -> Hertz {
        16_000_000.hz()
    }

    pub fn lptim1(&self) -> Hertz {
        self.lptim1
    }

    pub fn lptim2(&self) -> Hertz {
        self.lptim2
    }
}
