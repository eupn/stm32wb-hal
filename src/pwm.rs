//! # Pulse Width Modulation

use core::marker::PhantomData;
use core::mem;

use crate::hal;
use crate::stm32::{TIM1, TIM16, TIM2, TIM17};

use crate::gpio::gpioa::*;
use crate::gpio::gpiob::*;
use crate::gpio::{Alternate, Output, PushPull, AF1, AF14};
use crate::rcc::Rcc;
use crate::time::Hertz;

pub trait Pins<TIM> {
    const C1: bool = false;
    const C2: bool = false;
    const C3: bool = false;
    const C4: bool = false;
    type Channels;
}

macro_rules! pins_to_channels_mapping {
    ( $( $TIMX:ident: ( $($PINX:ident),+ ), ( $($ENCHX:ident),+ ), ( $($AF:ident),+ ); )+ ) => {
        $(
            #[allow(unused_parens)]
            impl Pins<$TIMX> for ($($PINX<Alternate<$AF, Output<PushPull>>>),+)
            {
                $(const $ENCHX: bool = true;)+
                type Channels = ($(Pwm<$TIMX, $ENCHX>),+);
            }
            //
            // #[allow(unused_parens)]
            // impl Pins<$TIMX> for ($($PINX<OpenDrain<$AF, Input<Floating>>>),+)
            // {
            //     $(const $ENCHX: bool = true;)+
            //     type Channels = ($(Pwm<$TIMX, $ENCHX>),+);
            // }
        )+
    };
}

// TODO: add other timers and channels
pins_to_channels_mapping! {
    // TIM1
    // TIM1: (PA8, PA9, PA10, PA11), (C1, C2, C3, C4), (AF1, AF1, AF1, AF1);
    // TIM1: (PA9, PA10, PA11), (C2, C3, C4), (AF1, AF1, AF1);
    // TIM1: (PA8, PA10, PA11), (C1, C3, C4), (AF1, AF1, AF1);
    // TIM1: (PA8, PA9, PA11), (C1, C2, C4), (AF1, AF1, AF1);
    // TIM1: (PA8, PA9, PA10), (C1, C2, C3), (AF1, AF1, AF1);
    // TIM1: (PA10, PA11), (C3, C4), (AF1, AF1);
    // TIM1: (PA9, PA11), (C2, C4), (AF1, AF1);
    // TIM1: (PA9, PA10), (C2, C3), (AF1, AF1);
    // TIM1: (PA8, PA11), (C1, C4), (AF1, AF1);
    // TIM1: (PA8, PA10), (C1, C3), (AF1, AF1);
    // TIM1: (PA8, PA9), (C1, C2), (AF1, AF1);
    // TIM1: (PA8), (C1), (AF1);
    // TIM1: (PA9), (C2), (AF1);
    // TIM1: (PA10), (C3), (AF1);
    // TIM1: (PA11), (C4), (AF1);

    // TIM2
    TIM2: (PA0, PA1, PA2, PA3), (C1, C2, C3, C4), (AF1, AF1, AF1, AF1);
    TIM2: (PA0), (C1), (AF1);
    TIM2: (PA1), (C2), (AF1);
    TIM2: (PA2), (C3), (AF1);
    TIM2: (PA3), (C4), (AF1);
    TIM2: (PA5), (C1), (AF1);
    TIM2: (PB3, PB10, PB11), (C2, C3, C4), (AF1, AF1, AF1);
    TIM2: (PB10), (C3), (AF1);
    TIM2: (PB11), (C4), (AF1);
    TIM2: (PA15), (C1), (AF1);

    // TIM16: (PB14), (C1), (AF14);
    // TIM16: (PB15), (C2), (AF14);
    // TIM16: (PA2), (C1), (AF14);
    // TIM16: (PA3), (C2), (AF14);
    // TIM16: (PB14, PB15), (C1, C2), (AF14, AF14);
    // TIM16: (PB14, PA3), (C1, C2), (AF14, AF14);
    // TIM16: (PA2, PB15), (C1, C2), (AF14, AF14);
    // TIM16: (PA2, PA3), (C1, C2), (AF14, AF14);
}

pub trait PwmExt1: Sized {
    fn pwm<PINS, T>(self, _: PINS, frequency: T, rcc: &mut Rcc) -> PINS::Channels
        where
            PINS: Pins<Self>,
            T: Into<Hertz>;
}

pub trait PwmExt2: Sized {
    fn pwm<PINS, T>(
        self,
        _: PINS,
        frequency: T,
        rcc: &mut Rcc,
    ) -> PINS::Channels
        where
            PINS: Pins<Self>,
            T: Into<Hertz>;
}

impl PwmExt1 for TIM1 {
    fn pwm<PINS, T>(self, _pins: PINS, freq: T, rcc: &mut Rcc) -> PINS::Channels
        where
            PINS: Pins<Self>,
            T: Into<Hertz>,
    {
        tim1(self, _pins, freq.into(), rcc)
    }
}

impl PwmExt1 for TIM2 {
    fn pwm<PINS, T>(self, _pins: PINS, freq: T, rcc: &mut Rcc) -> PINS::Channels
        where
            PINS: Pins<Self>,
            T: Into<Hertz>,
    {
        tim2(self, _pins, freq.into(), rcc)
    }
}

// impl PwmExt1 for TIM16 {
//     fn pwm<PINS, T>(self, _pins: PINS, freq: T, rcc: &mut Rcc) -> PINS::Channels
//         where
//             PINS: Pins<Self>,
//             T: Into<Hertz>,
//     {
//         tim16(self, _pins, freq.into(), rcc)
//     }
// }
//
// impl PwmExt2 for TIM2 {
//     fn pwm<PINS, T>(self, _pins: PINS, freq: T, rcc: &mut Rcc) -> PINS::Channels
//         where
//             PINS: Pins<Self>,
//             T: Into<Hertz>,
//     {
//         // TODO: check if this is really not needed (in the f1xx examples value
//         //       of remap is 0x0). if so, what's afio.mapr on wb55?
//         //
//         // mapr.mapr()
//         //     .modify(|_, w| unsafe { w.tim2_remap().bits(PINS::REMAP) });
//
//         tim2(self, _pins, freq.into(), rcc)
//     }
// }

pub struct Pwm<TIM, CHANNEL> {
    _channel: PhantomData<CHANNEL>,
    _tim: PhantomData<TIM>,
}

pub struct C1;
pub struct C2;
pub struct C3;
pub struct C4;

macro_rules! advanced_timer {
    ($($TIMX:ident: ($timX:ident, $timXen:ident, $timXrst:ident, $apben:ident, $apbrstr:ident, $psc_width:ident, $arr_width:ident),)+) => {
        $(
            fn $timX<PINS>(
                tim: $TIMX,
                _pins: PINS,
                freq: Hertz,
                rcc: &mut Rcc
            ) -> PINS::Channels
            where
                PINS: Pins<$TIMX>,
            {
                rcc.rb.$apben.modify(|_, w| w.$timXen().set_bit());
                rcc.rb.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                rcc.rb.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                if PINS::C1 {
                    tim.ccmr1_output().modify(|_, w| unsafe { w.oc1pe().set_bit().oc1m().bits(6) });
                }

                if PINS::C2 {
                    tim.ccmr1_output().modify(|_, w| unsafe { w.oc2pe().set_bit().oc2m().bits(6) });
                }

                if PINS::C3 {
                    tim.ccmr2_output().modify(|_, w| unsafe { w.oc3pe().set_bit().oc3m().bits(6) });
                }

                if PINS::C4 {
                    tim.ccmr2_output().modify(|_, w| unsafe { w.oc4pe().set_bit().oc4m().bits(6) });
                }

                let clk = rcc.clocks.pclk2().0;
                let freq = freq.0;
                let ticks = clk / freq;

                // maybe this is all u32? also, why no `- 1` vs `timer.rs`?
                let psc = ticks / (1 << 16);
                tim.psc.write(|w| unsafe { w.psc().bits(psc as $psc_width) });
                let arr = ticks / (psc + 1);
                tim.arr.write(|w| unsafe { w.arr().bits(arr as $arr_width) });

                // Only for the advanced control timer
                tim.bdtr.write(|w| w.moe().set_bit());
                tim.egr.write(|w| w.ug().set_bit());

                tim.cr1.write(|w| unsafe {
                    w.cms()
                        .bits(0b00)
                        .dir().clear_bit()
                        .opm().clear_bit()
                        .cen().set_bit()
                        .arpe().set_bit()
                });

                unsafe { mem::MaybeUninit::uninit().assume_init() }
            }

            pwm_channels! {
                $TIMX:  (C1, $arr_width, cc1e, ccr1, ccr1),
                        (C2, $arr_width, cc2e, ccr2, ccr2),
                        (C3, $arr_width, cc3e, ccr3, ccr3),
                        (C4, $arr_width, cc4e, ccr4, ccr4),
            }

        )+
    }
}

macro_rules! standard_timer {
    ($($TIMX:ident: ($timX:ident, $timXen:ident, $timXrst:ident, $apben:ident, $apbrstr:ident, $psc_width:ident),)+) => {
        $(
            fn $timX<PINS>(
                tim: $TIMX,
                _pins: PINS,
                freq: Hertz,
                rcc: &mut Rcc
            ) -> PINS::Channels
            where
                PINS: Pins<$TIMX>,
            {
                rcc.rb.$apben.modify(|_, w| w.$timXen().set_bit());
                rcc.rb.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                rcc.rb.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                if PINS::C1 {
                    tim.ccmr1_output().modify(|_, w| unsafe { w.oc1pe().set_bit().oc1m().bits(6) });
                }

                if PINS::C2 {
                    tim.ccmr1_output().modify(|_, w| unsafe { w.oc2pe().set_bit().oc2m().bits(6) });
                }

                if PINS::C3 {
                    tim.ccmr2_output().modify(|_, w| unsafe { w.oc3pe().set_bit().oc3m().bits(6) });
                }

                if PINS::C4 {
                    tim.ccmr2_output().modify(|_, w| unsafe { w.oc4pe().set_bit().oc4m().bits(6) });
                }

                let clk = rcc.clocks.pclk1().0;
                let freq = freq.0;
                let ticks = clk / freq;

                // maybe this is all u32? also, why no `- 1` vs `timer.rs`?
                let psc = ticks / (1 << 16);
                tim.psc.write(|w| unsafe { w.psc().bits(psc as $psc_width) });
                let arr = ticks / (psc + 1);
                tim.arr.write(|w| unsafe { w.bits(arr as u32) });

                tim.cr1.write(|w| unsafe {
                    w.cms()
                        .bits(0b00)
                        .dir().clear_bit()
                        .opm().clear_bit()
                        .cen().set_bit()
                        .arpe().set_bit()
                });

                unsafe { mem::MaybeUninit::uninit().assume_init() }
            }

            pwm_channels_hi_low! {
                $TIMX:  (C1, cc1e, ccr1, ccr1_h, ccr1_l),
                        (C2, cc2e, ccr2, ccr2_h, ccr2_l),
                        (C3, cc3e, ccr3, ccr3_h, ccr3_l),
                        (C4, cc4e, ccr4, ccr4_h, ccr4_l),
            }

        )+
    }
}

macro_rules! small_timer {
    ($($TIMX:ident: ($timX:ident, $timXen:ident, $timXrst:ident, $apben:ident, $apbrstr:ident, $psc_width:ident, $arr_width:ident),)+) => {
        $(
            fn $timX<PINS>(
                tim: $TIMX,
                _pins: PINS,
                freq: Hertz,
                rcc: &mut Rcc
            ) -> PINS::Channels
            where
                PINS: Pins<$TIMX>,
            {
                rcc.rb.$apben.modify(|_, w| w.$timXen().set_bit());
                rcc.rb.$apbrstr.modify(|_, w| w.$timXrst().set_bit());
                rcc.rb.$apbrstr.modify(|_, w| w.$timXrst().clear_bit());

                if PINS::C1 {
                    tim.ccmr1_output().modify(|_, w| unsafe { w.oc1pe().set_bit().oc1m().bits(6) });
                }

                // TODO: The uncommented lines are awaiting PAC updates to be valid.
                // if PINS::C2 {
                //     tim.ccmr1_output().modify(|_, w| unsafe { w.oc2pe().set_bit().oc2m().bits(6) });
                // }

                let clk = rcc.clocks.pclk1().0;
                let freq = freq.0;
                let ticks = clk / freq;

                // maybe this is all u32? also, why no `- 1` vs `timer.rs`?
                let psc = ticks / (1 << 16);
                tim.psc.write(|w| { w.psc().bits(psc as $psc_width) });
                let arr = ticks / (psc + 1);
                unsafe { tim.arr.write(|w| { w.arr().bits(arr as $arr_width) }); }

                tim.bdtr.write(|w| w.moe().set_bit());
                tim.egr.write(|w| w.ug().set_bit());

                /*
                tim.cr1.write(|w| {
                    w.opm().clear_bit()
                        .cen().set_bit()
                        .arpe().set_bit()
                });*/

                unsafe { mem::MaybeUninit::uninit().assume_init() }
            }

            pwm_channels! {
                $TIMX:  (C1, $arr_width, cc1e, ccr1, ccr1),
                // TODO: The uncommented line is awaiting PAC updates to be valid.
            //        (C2, $arr_width, cc2e, ccr2, ccr2),
            }

        )+
    }
}

macro_rules! pwm_channels {
    ($TIMX:ident: $(($channel:ident, $arr_width:ident, $ccXe:ident, $ccrX:ident, $ccr:ident),)+) => {
        $(
            impl hal::PwmPin for Pwm<$TIMX, $channel> {
                type Duty = $arr_width;

                #[inline(always)]
                fn disable(&mut self) {
                    unsafe { (*$TIMX::ptr()).ccer.modify(|_, w| w.$ccXe().clear_bit()) }
                }

                #[inline(always)]
                fn enable(&mut self) {
                    unsafe { (*$TIMX::ptr()).ccer.modify(|_, w| w.$ccXe().set_bit()) }
                }

                #[inline(always)]
                fn get_duty(&self) -> Self::Duty {
                    unsafe { (*$TIMX::ptr()).$ccrX.read().$ccr().bits() }
                }

                #[inline(always)]
                fn get_max_duty(&self) -> Self::Duty {
                    unsafe { (*$TIMX::ptr()).arr.read().arr().bits() }
                }

                #[inline(always)]
                fn set_duty(&mut self, duty: Self::Duty) {
                    unsafe { (*$TIMX::ptr()).$ccrX.write(|w| w.$ccr().bits(duty)) }
                }
            }
        )+
    }
}

macro_rules! pwm_channels_hi_low {
    ($TIMX:ident: $(($channel:ident, $ccXe:ident, $ccrX:ident, $ccrh:ident, $ccrl:ident),)+) => {
        $(
            impl hal::PwmPin for Pwm<$TIMX, $channel> {
                type Duty = u32;

                #[inline(always)]
                fn disable(&mut self) {
                    unsafe { (*$TIMX::ptr()).ccer.modify(|_, w| w.$ccXe().clear_bit()) }
                }

                #[inline(always)]
                fn enable(&mut self) {
                    unsafe { (*$TIMX::ptr()).ccer.modify(|_, w| w.$ccXe().set_bit()) }
                }

                #[inline(always)]
                fn get_duty(&self) -> Self::Duty {
                    let hi = unsafe { (*$TIMX::ptr()).$ccrX.read().$ccrh().bits() } as u32;
                    let low = unsafe { (*$TIMX::ptr()).$ccrX.read().$ccrl().bits() } as u32;
                    hi << 16 | low
                }

                #[inline(always)]
                fn get_max_duty(&self) -> Self::Duty {
                    let hi = unsafe { (*$TIMX::ptr()).arr.read().arr_h().bits() } as u32;
                    let low = unsafe { (*$TIMX::ptr()).arr.read().arr_l().bits() } as u32;
                    hi << 16 | low
                }

                #[inline(always)]
                fn set_duty(&mut self, duty: Self::Duty) {
                    let hi = (duty >> 16 & 0xffff) as u16;
                    let low = (duty & 0xffff) as u16;
                    unsafe { (*$TIMX::ptr()).$ccrX.write(|w| w.$ccrh().bits(hi).$ccrl().bits(low)) }
                }
            }
        )+
    }
}

advanced_timer! {
    TIM1: (tim1, tim1en, tim1rst, apb2enr, apb2rstr, u16, u16),
}

standard_timer! {
    TIM2: (tim2, tim2en, tim2rst, apb1enr1, apb1rstr1, u16),
}
//
// small_timer! {
//     TIM16: (tim16, tim16en, tim16rst, apb2enr, apb2rstr, u16, u16),
//     // TIM17: (tim17, tim17en, tim17rst, apb2enr, apb2rstr, u16, u16),
// }
