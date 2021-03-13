#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate panic_semihosting;
extern crate stm32wb_hal as hal;

use cortex_m_semihosting::hprintln;
use hal::flash::{FlashExt, FlashPage};
use hal::pac;
use hal::rcc::{ApbDivider, Config, HDivider, HseDivider, PllConfig, PllSrc, RccExt, SysClkSrc};
use hal::traits::flash::{Read, WriteErase};
use rt::entry;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();

    let clock_config = Config::new(SysClkSrc::Pll(PllSrc::Hse(HseDivider::NotDivided)))
        .cpu1_hdiv(HDivider::NotDivided)
        .cpu2_hdiv(HDivider::Div2)
        .apb1_div(ApbDivider::NotDivided)
        .apb2_div(ApbDivider::NotDivided)
        .pll_cfg(PllConfig {
            m: 2,
            n: 12,
            r: 3,
            q: Some(4),
            p: Some(3),
        });

    let mut flash_parts = dp.FLASH.constrain();
    rcc.apply_clock_config(clock_config, &mut flash_parts.acr);

    let mut flash = flash_parts
        .keyr
        .unlock_flash(
            &mut flash_parts.sr,
            &mut flash_parts.c2sr,
            &mut flash_parts.cr,
        )
        .unwrap();

    hprintln!("Started").ok();

    let page = FlashPage(127);
    let mut buf = [0u8; 4];
    flash.read(page.to_address(), &mut buf);

    if buf[0] == 0xFF {
        flash.erase_page(page).unwrap();
        flash.write(page.to_address(), &[1, 2, 3, 4]).unwrap();
        hprintln!("Has written to flash").ok();
    } else {
        hprintln!("Read flash: {:?}", buf).ok();
    }

    loop {
        cortex_m::asm::wfi();
    }
}
