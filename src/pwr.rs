/// Enables or disables USB power supply.
pub fn set_usb(enable: bool) {
    let pwr = unsafe { &*stm32wb_pac::PWR::ptr() };
    pwr.cr2.modify(|_, w| w.usv().bit(enable));
}
