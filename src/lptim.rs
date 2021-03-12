//! Low-Power Timer (LPTIM) support.

use crate::hal;
use crate::pac::{LPTIM1, LPTIM2};
use crate::rcc::Rcc;
use crate::time::{Hertz, MicroSecond};

use cast::{u32, u64};
use core::convert::TryFrom;
use core::marker::PhantomData;
use void::Void;

mod sealed {
    pub trait Sealed {}
}

/// Low-Power Timer counting in one-shot mode.
pub enum OneShot {}

/// Low-Power Timer counting in periodic mode.
pub enum Periodic {}

impl sealed::Sealed for OneShot {}
impl sealed::Sealed for Periodic {}

/// Marker trait for counter directions.
pub trait CountMode: sealed::Sealed {}

impl CountMode for OneShot {}
impl CountMode for Periodic {}

/// Interrupt enable flags.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Interrupts {
    /// Encoder direction change to down.
    pub enc_dir_down: bool,
    /// Encoder direction change to up.
    pub enc_dir_up: bool,
    /// ARR register update successful.
    pub autoreload_update_ok: bool,
    /// CMP register update successful.
    pub compare_update_ok: bool,
    /// Valid edge on ext. trigger input.
    pub ext_trig: bool,
    /// ARR register matches current CNT value.
    pub autoreload_match: bool,
    /// CMP register matches current CNT value.
    pub compare_match: bool,
}

macro_rules! impl_lptim {
    ($LPTIM:ty, $lptim:ident, $apb1enr:ident, $lptimen:ident, $apb1rst:ident, $lptimrst:ident) => {
        pub mod $lptim {
            use super::*;

            /// Low-Power Timer (`LPTIM`).
            ///
            /// The Low-Power Timer is a 16-bit timer with a prescaler of up to 128. It can run off of the APB1,
            /// LSI, HSI16, or LSE clocks. With LSE, the slowest clock at 32.768 kHz, this results in a maximum
            /// timeout of 256 seconds, or 4 minutes and 16 seconds.
            ///
            /// The timer can be initialized either in one-shot mode or in periodic mode, using `init_oneshot`
            /// or `init_periodic` respectively. In periodic mode, the embedded-hal `Periodic` marker trait is
            /// implemented and the `CountDown` implementation uses `Hertz` as the time unit. In one-shot mode,
            /// the `CountDown` implementation instead uses `MicroSecond`, allowing for a multi-second timeout
            /// to be configured (with the tradeoff being a larger code size due to use of 64-bit arithmetic).
            pub struct LpTimer<M: CountMode> {
                lptim: $LPTIM,
                input_freq: Hertz,
                _mode: PhantomData<M>,
            }

            impl LpTimer<Periodic> {
                /// Initializes the Low-Power Timer in periodic mode.
                ///
                /// The timer needs to be started by calling `.start(freq)`.
                pub fn init_periodic(lptim: $LPTIM, rcc: &mut Rcc) -> Self {
                    Self::init(lptim, rcc)
                }
            }

            impl LpTimer<OneShot> {
                /// Initializes the Low-Power Timer in one-shot mode.
                ///
                /// The timer needs to be started by calling `.start(freq)`.
                pub fn init_oneshot(lptim: $LPTIM, rcc: &mut Rcc) -> Self {
                    Self::init(lptim, rcc)
                }
            }

            impl<M: CountMode> LpTimer<M> {
                fn init(lptim: $LPTIM, rcc: &mut Rcc) -> Self {
                    let input_freq = rcc.clocks.$lptim();

                    rcc.rb.$apb1enr.modify(|_, w| w.$lptimen().set_bit());

                    rcc.rb.$apb1rst.modify(|_, w| w.$lptimrst().set_bit());
                    rcc.rb.$apb1rst.modify(|_, w| w.$lptimrst().clear_bit());

                    Self {
                        lptim,
                        input_freq,
                        _mode: PhantomData,
                    }
                }

                /// Disables the timer and configures it so that starting it will make it fire at the given
                /// frequency.
                fn configure(&mut self, conf: TimeConf) {
                    // Disable the timer. The prescaler can only be changed while it's disabled.
                    self.lptim.cr.write(|w| w.enable().clear_bit());

                    self.lptim
                        .cfgr
                        .write(|w| unsafe { w.presc().bits(conf.psc_encoded).timout().set_bit() });

                    self.lptim.cr.write(|w| w.enable().set_bit());

                    // "After setting the ENABLE bit, a delay of two counter clock is needed before the LPTIM is
                    // actually enabled."
                    // The slowest LPTIM clock source is LSE at 32768 Hz, the fastest CPU clock is ~80 MHz. At
                    // these conditions, one cycle of the LPTIM clock takes 2500 CPU cycles, so sleep for 5000.
                    cortex_m::asm::delay(5000);

                    // ARR can only be changed while the timer is *en*abled
                    self.lptim.arr.write(|w| unsafe { w.arr().bits(conf.arr) });
                }

                /// Disables and destructs the timer, returning the raw `LPTIM` peripheral.
                pub fn free(self) -> $LPTIM {
                    self.lptim.cr.reset();
                    self.lptim
                }

                /// Disables the timer and enables the given interrupts.
                pub fn enable_interrupts(&mut self, interrupts: Interrupts) {
                    // IER can only be modified when the timer is disabled
                    self.lptim.cr.reset();
                    self.lptim.ier.modify(|_, w| {
                        if interrupts.enc_dir_down {
                            w.downie().set_bit();
                        }
                        if interrupts.enc_dir_up {
                            w.upie().set_bit();
                        }
                        if interrupts.autoreload_update_ok {
                            w.arrokie().set_bit();
                        }
                        if interrupts.compare_update_ok {
                            w.cmpokie().set_bit();
                        }
                        if interrupts.ext_trig {
                            w.exttrigie().set_bit();
                        }
                        if interrupts.autoreload_match {
                            w.arrmie().set_bit();
                        }
                        if interrupts.compare_match {
                            w.cmpmie().set_bit();
                        }
                        w
                    })
                }

                /// Clears the given interrupt.
                pub fn clear_interrupts(&mut self, interrupts: Interrupts) {
                    self.lptim.icr.write(|w| {
                        if interrupts.enc_dir_down {
                            w.downcf().set_bit();
                        }
                        if interrupts.enc_dir_up {
                            w.upcf().set_bit();
                        }
                        if interrupts.autoreload_update_ok {
                            w.arrokcf().set_bit();
                        }
                        if interrupts.compare_update_ok {
                            w.cmpokcf().set_bit();
                        }
                        if interrupts.ext_trig {
                            w.exttrigcf().set_bit();
                        }
                        if interrupts.autoreload_match {
                            w.arrmcf().set_bit();
                        }
                        if interrupts.compare_match {
                            w.cmpmcf().set_bit();
                        }
                        w
                    })
                }

                /// Disables the timer and disables the given interrupts.
                pub fn disable_interrupts(&mut self, interrupts: Interrupts) {
                    // IER can only be modified when the timer is disabled
                    self.lptim.cr.reset();
                    self.lptim.ier.modify(|_, w| {
                        if interrupts.enc_dir_down {
                            w.downie().clear_bit();
                        }
                        if interrupts.enc_dir_up {
                            w.upie().clear_bit();
                        }
                        if interrupts.autoreload_update_ok {
                            w.arrokie().clear_bit();
                        }
                        if interrupts.compare_update_ok {
                            w.cmpokie().clear_bit();
                        }
                        if interrupts.ext_trig {
                            w.exttrigie().clear_bit();
                        }
                        if interrupts.autoreload_match {
                            w.arrmie().clear_bit();
                        }
                        if interrupts.compare_match {
                            w.cmpmie().clear_bit();
                        }
                        w
                    })
                }
            }

            impl crate::hal::timer::CountDown for LpTimer<Periodic> {
                type Time = Hertz;

                fn start<T>(&mut self, freq: T)
                where
                    T: Into<Hertz>,
                {
                    self.configure(TimeConf::calculate_freq(self.input_freq, freq.into()));

                    // Start LPTIM in continuous mode.
                    self.lptim
                        .cr
                        .write(|w| w.enable().set_bit().cntstrt().set_bit());
                }

                fn wait(&mut self) -> nb::Result<(), Void> {
                    if self.lptim.isr.read().arrm().bit_is_clear() {
                        Err(nb::Error::WouldBlock)
                    } else {
                        self.lptim.icr.write(|w| w.arrmcf().set_bit());
                        Ok(())
                    }
                }
            }

            impl hal::timer::Periodic for LpTimer<Periodic> {}

            impl hal::timer::CountDown for LpTimer<OneShot> {
                type Time = MicroSecond;

                fn start<T>(&mut self, period: T)
                where
                    T: Into<MicroSecond>,
                {
                    self.configure(TimeConf::calculate_period(self.input_freq, period.into()));

                    // Start LPTIM in one-shot mode.
                    self.lptim
                        .cr
                        .write(|w| w.enable().set_bit().sngstrt().set_bit());
                }

                fn wait(&mut self) -> nb::Result<(), Void> {
                    if self.lptim.isr.read().arrm().bit_is_clear() {
                        Err(nb::Error::WouldBlock)
                    } else {
                        self.lptim.icr.write(|w| w.arrmcf().set_bit());
                        Ok(())
                    }
                }
            }

            #[derive(Copy, Clone)]
            struct TimeConf {
                psc_encoded: u8,
                arr: u16,
            }

            impl TimeConf {
                const ARR_MAX: u16 = u16::MAX;

                /// Calculates prescaler and autoreload value for producing overflows at a rate of
                /// `output_freq`.
                fn calculate_freq(input_freq: Hertz, output_freq: Hertz) -> Self {
                    // Fi  = Frequency of input clock
                    // Fo  = Output frequency (frequency of timer overflows, using ARR)
                    // psc = prescaler (must be power of two in range 1..=128)
                    // We know Fi and Fo, and want to know psc and ARR.
                    //
                    // The timer works like this:
                    // Fo  = (Fi / psc) / ARR
                    //
                    // Therefore:
                    // Fo * ARR = Fi / psc
                    // Fo * ARR * psc = Fi
                    // ARR = (Fi / Fo) / psc
                    // psc = (Fi / Fo) / ARR
                    //
                    // We first calculate `psc` by assuming the largest `ARR` value, and round the result to the
                    // next power of two. If that's > 128, the chosen frequency is too slow for the timer and
                    // we panic. Otherwise we use that `psc` to calculate the real `ARR`.

                    // Add `ARR_MAX - 1` to round the result upwards
                    let psc = ((input_freq.0 / output_freq.0) + (u32(Self::ARR_MAX) - 1))
                        / u32(Self::ARR_MAX);
                    let psc = psc.next_power_of_two(); // always >= 1
                    assert!(psc <= 128);

                    // This calculation must be in u16 range because we assume the max. ARR value above ^
                    let arr = u16::try_from((input_freq.0 / output_freq.0) / psc).unwrap();

                    // PSC encoding is N where `psc = 2^N`
                    let psc_encoded = psc.trailing_zeros() as u8;

                    Self { psc_encoded, arr }
                }

                /// Calculates prescaler and autoreload value for producing overflows after every
                /// `output_period`.
                fn calculate_period(input_freq: Hertz, output_period: MicroSecond) -> Self {
                    // Here, the `output_period` can be very long, resulting in an output frequency of < 1 Hz.

                    // Fi  = Frequency of input clock
                    // Fo  = Output frequency (frequency of timer overflows, using ARR)
                    // Po  = 1 / Fo = Output Period
                    // psc = prescaler (must be power of two in range 1..=128)
                    // We know Fi and Fo, and want to know psc and ARR.
                    //
                    // The timer works like this:
                    // Fo  = 1 / Po = (Fi / psc) / ARR
                    //
                    // Therefore:
                    // ARR / Po = Fi / psc
                    // (ARR * psc) / Po = Fi
                    // ARR * psc = Fi * Po
                    // ARR = (Fi * Po) / psc
                    // psc = (Fi * Po) / ARR
                    //
                    // We first calculate `psc` by assuming the largest `ARR` value, and round the result to the
                    // next power of two. If that's > 128, the chosen period is too long for the timer and we
                    // panic. Otherwise we use that `psc` to calculate the real `ARR`.

                    // First, calculate the product `Fi * Po`. Since `output_period` is in µs, we have to divide
                    // it by 1_000_000 to get seconds, without losing much precision. We can divide either of
                    // the multiplicants, or the resulting product. Dividing the resulting product results in
                    // the least amount of rouding error, but might require 64-bit multiplication and division,
                    // which is very expensive. Dividing either of the multiplicands by 1_000_000 can easily
                    // result in significant rounding error that makes this API useless.
                    let fi_po = u32(u64(input_freq.0) * u64(output_period.0) / 1_000_000).unwrap();
                    // Add `ARR_MAX - 1` to round the result upwards
                    let psc = (fi_po + (u32(Self::ARR_MAX) - 1)) / u32(Self::ARR_MAX);
                    assert!(psc > 0); // if 0, the output period is too short to be produced from input_freq
                    let psc = psc.next_power_of_two(); // always >= 1
                    assert!(psc <= 128); // if > 128, the output period is too long to be produced from input_freq

                    // This calculation must be in u16 range because we assume the max. ARR value above ^
                    let arr = (fi_po / psc) as u16;

                    // PSC encoding is N where `psc = 2^N`
                    let psc_encoded = psc.trailing_zeros() as u8;

                    Self { psc_encoded, arr }
                }
            }
        }
    };
}

impl_lptim!(LPTIM1, lptim1, apb1enr1, lptim1en, apb1rstr1, lptim1rst);
impl_lptim!(LPTIM2, lptim2, apb1enr2, lptim2en, apb1rstr2, lptim2rst);
