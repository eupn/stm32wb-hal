use crate::pac;

/// Enables or disables USB power supply.
pub fn set_usb(enable: bool) {
    let pwr = unsafe { &*pac::PWR::ptr() };
    pwr.cr2.modify(|_, w| w.usv().bit(enable));
}

/// Enables or disables CPU2 Cortex-M0 radio co-processor.
pub fn set_cpu2(enabled: bool) {
    let pwr = unsafe { &*pac::PWR::ptr() };
    pwr.cr4.modify(|_, w| w.c2boot().bit(enabled))
}

/// Enables or disables access to the backup domain.
pub fn set_backup_access(enabled: bool) {
    let pwr = unsafe { &*pac::PWR::ptr() };

    // ST: write twice the value to flush the APB-AHB bridge to ensure the bit is written
    pwr.cr1.modify(|_, w| w.dbp().bit(enabled));
    pwr.cr1.modify(|_, w| w.dbp().bit(enabled));
}
