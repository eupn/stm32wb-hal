/// Enables or disables USB power supply.
pub fn set_usb(enable: bool) {
    let pwr = unsafe { &*stm32wb_pac::PWR::ptr() };
    pwr.cr2.modify(|_, w| w.usv().bit(enable));
}

/// Enables or disables CPU2 Cortex-M0 radio co-processor.
pub fn set_cpu2(enabled: bool) {
    let pwr = unsafe { &*stm32wb_pac::PWR::ptr() };
    pwr.cr4.modify(|_, w| w.c2boot().bit(enabled))
}
