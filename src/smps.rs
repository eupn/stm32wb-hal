//! Switch-Mode Power Supply (SMPS) module

use crate::pac;

pub struct Smps {}

impl Smps {
    pub fn enable() {
        let pwr = unsafe { pac::Peripherals::steal().PWR };
        pwr.cr5.modify(|_, w| w.sdeb().set_bit())
    }

    pub fn disable() {
        let pwr = unsafe { pac::Peripherals::steal().PWR };
        pwr.cr5.modify(|_, w| w.sdeb().clear_bit())
    }

    pub fn is_enabled() -> bool {
        let pwr = unsafe { pac::Peripherals::steal().PWR };
        pwr.cr5.read().sdeb().bit()
    }
}
