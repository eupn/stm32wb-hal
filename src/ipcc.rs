use stm32wb_pac::IPCC;
use crate::rcc::Rcc;

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum IpccChannel {
    Channel1 = 0x00000001,
    Channel2 = 0x00000002,
    Channel3 = 0x00000004,
    Channel4 = 0x00000008,
    Channel5 = 0x00000010,
    Channel6 = 0x00000020,
}

pub struct Ipcc {
    rb: IPCC,
}

impl Ipcc {
    pub fn init(&mut self, rcc: &mut Rcc) {
        rcc.set_ipcc(true);

        // Enable IPCC interrupts
        self.rb.c1cr.modify(|_, w| w.rxoie().set_bit().txfie().set_bit());
        unsafe {
            cortex_m::peripheral::NVIC::unmask(stm32wb_pac::interrupt::IPCC_C1_RX_IT);
            cortex_m::peripheral::NVIC::unmask(stm32wb_pac::interrupt::IPCC_C1_TX_IT);
        }
    }
}

/// Extension trait that constrains the `IPCC` peripheral
pub trait IpccExt {
    /// Constrains the `IPCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Ipcc;
}

impl IpccExt for IPCC {
    fn constrain(self) -> Ipcc {
        Ipcc {
            rb: self,
        }
    }
}