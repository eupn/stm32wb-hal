use super::mux::*;

#[derive(Debug)]
pub struct Config {
    pub(crate) lse: bool,
    pub(crate) lsi1: bool,

    pub(crate) sysclk_src: SysClkSrc,

    pub(crate) pll_cfg: PllConfig,

    pub(crate) apb1_div: ApbDivider,
    pub(crate) apb2_div: ApbDivider,

    pub(crate) cpu1_hdiv: HDivider,
    pub(crate) cpu2_hdiv: HDivider,
    pub(crate) hclk_hdiv: HDivider,

    pub(crate) usb_src: UsbClkSrc,
}

impl Default for Config {
    /// From MSI, 4MHz, no PLL. No dividers applied.
    /// SYSCLK = 4 MHz, HCLK = 4MHz, CPU1 = CPU2 = 4MHz, APB1 = APB2 = 4MHz
    fn default() -> Self {
        Config {
            lse: false,
            lsi1: false,
            sysclk_src: SysClkSrc::Hsi,
            pll_cfg: PllConfig::default(),
            apb1_div: ApbDivider::NotDivided,
            apb2_div: ApbDivider::NotDivided,
            cpu1_hdiv: HDivider::NotDivided,
            cpu2_hdiv: HDivider::NotDivided,
            hclk_hdiv: HDivider::NotDivided,
            usb_src: UsbClkSrc::default(),
        }
    }
}

impl Config {
    pub fn new(mux: SysClkSrc) -> Self {
        Config::default().clock_src(mux)
    }

    pub fn pll() -> Self {
        Config::default()
            .clock_src(SysClkSrc::Pll(PllSrc::Msi(MsiRange::default())))
            .pll_cfg(PllConfig::default())
    }

    pub fn hsi() -> Self {
        Config::default().clock_src(SysClkSrc::Hsi)
    }

    pub fn hse_sys(hse_divider: HseDivider) -> Self {
        Config::default().clock_src(SysClkSrc::HseSys(hse_divider))
    }

    pub fn clock_src(mut self, mux: SysClkSrc) -> Self {
        self.sysclk_src = mux;
        self
    }

    pub fn pll_cfg(mut self, cfg: PllConfig) -> Self {
        self.pll_cfg = cfg;
        self
    }

    pub fn apb1_div(mut self, div: ApbDivider) -> Self {
        self.apb1_div = div;
        self
    }

    pub fn apb2_div(mut self, div: ApbDivider) -> Self {
        self.apb2_div = div;
        self
    }

    pub fn cpu1_hdiv(mut self, div: HDivider) -> Self {
        self.cpu1_hdiv = div;
        self
    }

    pub fn cpu2_hdiv(mut self, div: HDivider) -> Self {
        self.cpu2_hdiv = div;
        self
    }

    pub fn usb_src(mut self, src: UsbClkSrc) -> Self {
        self.usb_src = src;
        self
    }

    pub fn with_lse(mut self) -> Self {
        self.lse = true;
        self
    }

    pub fn with_lsi1(mut self) -> Self {
        self.lsi1 = true;
        self
    }
}

#[derive(Debug, Clone)]
pub enum MsiRange {
    #[doc = "range 0 around 100 kHz"]
    RANGE100K = 0,
    #[doc = "range 1 around 200 kHz"]
    RANGE200K = 1,
    #[doc = "range 2 around 400 kHz"]
    RANGE400K = 2,
    #[doc = "range 3 around 800 kHz"]
    RANGE800K = 3,
    #[doc = "range 4 around 1 MHz"]
    RANGE1M = 4,
    #[doc = "range 5 around 2 MHz"]
    RANGE2M = 5,
    #[doc = "range 6 around 4 MHz"]
    RANGE4M = 6,
    #[doc = "range 7 around 8 MHz"]
    RANGE8M = 7,
    #[doc = "range 8 around 16 MHz"]
    RANGE16M = 8,
    #[doc = "range 9 around 24 MHz"]
    RANGE24M = 9,
    #[doc = "range 10 around 32 MHz"]
    RANGE32M = 10,
    #[doc = "range 11 around 48 MHz"]
    RANGE48M = 11,
}

impl Default for MsiRange {
    fn default() -> Self {
        MsiRange::RANGE4M
    }
}

/// HSE input divider.
#[derive(Debug, Clone)]
pub enum HseDivider {
    NotDivided,
    Div2,
}

/// PLL configuration.
#[derive(Debug, Clone)]
pub struct PllConfig {
    pub m: u8,
    pub n: u8,
    pub r: u8,
    pub q: Option<u8>,
    pub p: Option<u8>,
}

impl Default for PllConfig {
    fn default() -> Self {
        PllConfig {
            m: 1,
            n: 8,
            r: 2,
            q: None,
            p: None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ApbDivider {
    NotDivided = 0b000,
    Div2 = 0b100,
    Div4 = 0b101,
    Div8 = 0b110,
    Div16 = 0b111,
}

impl ApbDivider {
    pub fn divisor(&self) -> u32 {
        match self {
            ApbDivider::NotDivided => 1,
            ApbDivider::Div2 => 2,
            ApbDivider::Div4 => 4,
            ApbDivider::Div8 => 8,
            ApbDivider::Div16 => 16,
        }
    }
}

/// CPU1, CPU2 HPRE (prescaler).
/// RM0434 page 230.
#[derive(Debug, Copy, Clone)]
pub enum HDivider {
    NotDivided = 0,
    Div2 = 0b1000,
    Div3 = 0b0001,
    Div4 = 0b1001,
    Div5 = 0b0010,
    Div6 = 0b0101,
    Div10 = 0b0110,
    Div8 = 0b1010,
    Div16 = 0b1011,
    Div32 = 0b0111,
    Div64 = 0b1100,
    Div128 = 0b1101,
    Div256 = 0b1110,
    Div512 = 0b1111,
}

impl HDivider {
    /// Returns division value
    pub fn divisor(&self) -> u32 {
        match self {
            HDivider::NotDivided => 1,
            HDivider::Div2 => 2,
            HDivider::Div3 => 3,
            HDivider::Div4 => 4,
            HDivider::Div5 => 5,
            HDivider::Div6 => 6,
            HDivider::Div10 => 10,
            HDivider::Div8 => 8,
            HDivider::Div16 => 16,
            HDivider::Div32 => 32,
            HDivider::Div64 => 64,
            HDivider::Div128 => 128,
            HDivider::Div256 => 256,
            HDivider::Div512 => 512,
        }
    }
}

#[derive(Debug)]
pub enum StopWakeupClock {
    MSI = 0,
    HSI16 = 1,
}