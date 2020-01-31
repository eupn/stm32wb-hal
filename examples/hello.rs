#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_halt;
extern crate stm32wb_hal as hal;

use cortex_m_semihosting::hprintln;
use rt::entry;

#[entry]
fn main() -> ! {
    hprintln!("Hello, STM32WB55!").unwrap();

    loop {}
}
