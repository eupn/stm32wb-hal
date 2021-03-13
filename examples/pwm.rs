//! PWM LED smooth blinky example

#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate panic_halt;

use cortex_m_rt::entry;
use stm32wb_hal::flash::FlashExt;
use stm32wb_hal::rcc::{ApbDivider, Config, HDivider, HseDivider, PllConfig, PllSrc, SysClkSrc};
use stm32wb_hal::{delay, pac, prelude::*};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let syst = cp.SYST;
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

    let mut rcc = rcc.apply_clock_config(clock_config, &mut dp.FLASH.constrain().acr);
    let mut delay = crate::delay::Delay::new(syst, rcc.clocks.clone());
    let mut gpioa = dp.GPIOA.split(&mut rcc);

    // TIM2, led output
    let c1 = gpioa
        .pa5
        .into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper)
        .into_af1(&mut gpioa.moder, &mut gpioa.afrl);

    let mut pwm = PwmExt1::pwm(dp.TIM2, c1, 1.khz(), &mut rcc);

    let max = pwm.get_max_duty();
    pwm.enable();

    // Gradually go back and forth from max to 0 duty
    // to get a smooth LED blink
    let mut up = true;
    let mut duty = max;
    let blink_speed = max / 8; // tune this to have different blink speed
    loop {
        if duty <= 0 {
            duty = 0;
            up = true;
        } else if duty >= max {
            duty = max;
            up = false;
        }

        if up {
            duty += blink_speed;
        } else {
            duty -= blink_speed;
        }

        pwm.set_duty(duty);
        delay.delay_ms(max / 1000); // divide by 1000 is due to 1 kHz PWM frequency
    }
}
