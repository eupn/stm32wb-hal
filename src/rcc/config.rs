use crate::time::Hertz;

pub struct Config {
    pub(crate) sysclk_src: SysClkSrc,

    pub(crate) pll_cfg: PllConfig,

    pub(crate) apb1_div: ApbDivider, // Max 64 MHz
    pub(crate) apb2_div: ApbDivider, // Max 64 MHz

    pub(crate) cpu1_hdiv: HDivider, // Max 64 MHz
    pub(crate) cpu2_hdiv: HDivider, // Max 32 MHz
}

impl Default for Config {
    /// From MSI, 4MHz, no PLL. No dividers applied.
    /// SYSCLK = 4 MHz, HCLK = 4MHz, CPU1 = CPU2 = 4MHz, APB1 = APB2 = 4MHz
    fn default() -> Self {
        Config {
            sysclk_src: SysClkSrc::Hsi,
            pll_cfg: PllConfig::default(),
            apb1_div: ApbDivider::NotDivided,
            apb2_div: ApbDivider::NotDivided,
            cpu1_hdiv: HDivider::NotDivided,
            cpu2_hdiv: HDivider::NotDivided
        }
    }
}

impl Config {
    pub fn new(mux: SysClockSrc) -> Self {
        Config::default().clock_src(mux)
    }

    pub fn pll() -> Self {
        Config::default().clock_src(SysClkSrc::Pll(PllConfig::default()))
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
}

/// System clock (SYSCLK) source selection.
pub enum SysClkSrc {
    /// Multi-speed internal RC oscillator
    Msi(MsiRange),

    /// 16 MHz internal RC
    Hsi,

    /// Use HSE directly, without PLL.
    HseSys(HseDivider),

    /// Use PLL.
    Pll(PllConfig),
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
pub enum HseDivider {
    NotDivided,
    Div2,
}

/// PLL configuration.
#[derive(Clone, Copy)]
pub struct PllConfig {
    source: PllSrcMux,

    m: u8,
    n: u8,
    r: u8,
    q: Option<u8>,
    p: Option<u8>,
}

impl Default for PllConfig {
    fn default() -> Self {
        PllConfig {
            source: PllSrcMux::Msi(MsiRange::default()),
            m: 1,
            n: 8,
            r: 2,
            q: Some(2),
            p: Some(2),
        }
    }
}

/// PLL input frequency source.
pub enum PllSrcMux {
    Msi(MsiRange),
    Hsi,
    Hse(HseDivider),
}

#[derive(Debug)]
pub enum ApbDivider {
    NotDivided = 1,
    Div2 = 2,
    Div4 = 4,
    Div8 = 8,
    Div16 = 16,
}

/// CPU1, CPU2 HPRE (prescaler).
/// RM0434 page 230.
#[derive(Debug)]
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