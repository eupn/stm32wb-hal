//! Serial Peripheral Interface (SPI) bus

use crate::stm32::{SPI1, SPI2};

use crate::gpio::gpioa::{PA1, PA11, PA12, PA14, PA15, PA4, PA5, PA6, PA7, PA9};
use crate::gpio::gpiob::{PB10, PB12, PB13, PB14, PB15, PB2, PB3, PB4, PB5, PB9};
use crate::gpio::gpioc::{PC1, PC2, PC3};
use crate::gpio::gpiod::{PD0, PD1, PD3, PD4};
use crate::gpio::{Alternate, Output, PushPull, AF3, AF5};
use crate::hal;
// use crate::hal::blocking::spi::{Read, Write, WriteRead};
use crate::rcc::Rcc;
use crate::time::Hertz;

use core::ptr;

pub use crate::hal::spi::{Mode, Phase, Polarity, MODE_0, MODE_1, MODE_2, MODE_3};

/// SPI error
#[derive(Debug)]
pub enum Error {
    Busy,
    FrameError,
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
    #[doc(hidden)]
    _Extensible,
}

pub trait Pins<SPI> {}
pub trait PinSck<SPI> {}
pub trait PinMiso<SPI> {}
pub trait PinMosi<SPI> {}
pub trait PinNss<SPI> {}

impl<SPI, SCK, MISO, MOSI> Pins<SPI> for (SCK, MISO, MOSI)
where
    SCK: PinSck<SPI>,
    MISO: PinMiso<SPI>,
    MOSI: PinMosi<SPI>,
{
    // fn setup(&self) {
    //     // self.0.setup();
    //     // self.1.setup();
    //     // self.2.setup();
    // }
}

/// A filler type for when the SCK pin is unnecessary
pub struct NoSck;
/// A filler type for when the Miso pin is unnecessary
pub struct NoMiso;
/// A filler type for when the Mosi pin is unnecessary
pub struct NoMosi;
/// A filler type for when the Nss pin is unnecessary
pub struct NoNss;

macro_rules! pins {
    ($($SPIX:ty:
        SCK: [$([$SCK:ident, $ALTMODESCK:path]),* $(,)?]
        MISO: [$([$MISO:ident, $ALTMODEMISO:path]),* $(,)?]
        MOSI: [$([$MOSI:ident, $ALTMODEMOSI:path]),* $(,)?]
        NSS: [$([$NSS:ident, $ALTMODENSS:path]),* $(,)?])+) => {
        $(
            $(
                impl PinSck<$SPIX> for $SCK::<Alternate<$ALTMODESCK, Output<PushPull>>> {
                }
            )*
            $(
                impl PinMiso<$SPIX> for $MISO::<Alternate<$ALTMODEMISO, Output<PushPull>>> {
                }
            )*
            $(
                impl PinMosi<$SPIX> for $MOSI::<Alternate<$ALTMODEMOSI, Output<PushPull>>> {
                }
            )*
            $(
                impl PinNss<$SPIX> for $NSS::<Alternate<$ALTMODENSS, Output<PushPull>>> {
                }
            )*
        )+
    }
}

impl PinSck<SPI1> for NoSck {}
impl PinMiso<SPI1> for NoMiso {}
impl PinMosi<SPI1> for NoMosi {}
impl PinNss<SPI1> for NoNss {}
impl PinSck<SPI2> for NoSck {}
impl PinMiso<SPI2> for NoMiso {}
impl PinMosi<SPI2> for NoMosi {}
impl PinNss<SPI2> for NoNss {}

pins! {
    SPI1:
        SCK: [
            [PA1, AF5],
            [PA5, AF5],
            [PB3, AF5],
        ]
        MISO: [
            [PA6, AF5],
            [PA11, AF5],
            [PB4, AF5],
        ]
        MOSI: [
            [PA7, AF5],
            [PA12, AF5],
            [PB5, AF5],
        ]
        NSS: [
            [PA4, AF5],
            [PA14, AF5],
            [PA15, AF5],
            [PB2, AF5],
        ]
}

pins! {
    SPI2:
        SCK: [
            [PA9, AF5],
            [PB10, AF5],
            [PB13, AF5],
            [PD1, AF5],
            [PD3, AF3],
        ]
        MISO: [
            [PB14, AF5],
            [PC2, AF5],
            [PD3, AF5],
            [PD4, AF5],
            [PC1, AF3],
        ]
        MOSI: [
            [PB15, AF5],
            [PB3, AF5],
            [PC3, AF5],
        ]
        NSS: [
            [PB9, AF5],
            [PB12, AF5],
            [PD0, AF5],
        ]
}

#[derive(Debug)]
pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

pub trait SpiExt<SPI>: Sized {
    fn spi<PINS, T>(self, pins: PINS, mode: Mode, freq: T, rcc: &mut Rcc) -> Spi<SPI, PINS>
    where
        PINS: Pins<SPI>,
        T: Into<Hertz>;
}

macro_rules! spi {
    ($($SPIX:ident: ($spiX:ident, $apbXenr:ident, $spiXen:ident, $pclkX:ident),)+) => {
        $(
            impl<PINS> Spi<$SPIX, PINS> {
                pub fn $spiX<T>(
                    spi: $SPIX,
                    pins: PINS,
                    mode: Mode,
                    freq: T,
                    rcc: &mut Rcc
                ) -> Self
                where
                PINS: Pins<$SPIX>,
                T: Into<Hertz>
                {
                    // Enable clock for SPI
                    rcc.rb.$apbXenr.modify(|_, w| w.$spiXen().set_bit());

                    // disable SS output
                    spi.cr2.write(|w| w.ssoe().clear_bit());

                    let spi_freq = freq.into().0;
                    let apb_freq = rcc.clocks.$pclkX().0;
                    let br = match apb_freq / spi_freq {
                        0 => unreachable!(),
                        1..=2 => 0b000,
                        3..=5 => 0b001,
                        6..=11 => 0b010,
                        12..=23 => 0b011,
                        24..=47 => 0b100,
                        48..=95 => 0b101,
                        96..=191 => 0b110,
                        _ => 0b111,
                    };

                    // mstr: master configuration
                    // lsbfirst: MSB first
                    // ssm: enable software slave management (NSS pin free for other uses)
                    // ssi: set nss high = master mode
                    // dff: 8 bit frames
                    // bidimode: 2-line unidirectional
                    // spe: enable the SPI bus
                    #[allow(unused)]
                    spi.cr1.write(|w| unsafe {
                        w.cpha()
                            .bit(mode.phase == Phase::CaptureOnSecondTransition)
                            .cpol()
                            .bit(mode.polarity == Polarity::IdleHigh)
                            .mstr()
                            .set_bit()
                            .br()
                            .bits(br)
                            .lsbfirst()
                            .clear_bit()
                            .ssm()
                            .set_bit()
                            .ssi()
                            .set_bit()
                            .rxonly()
                            .clear_bit()
                            .dff()
                            .clear_bit()
                            .bidimode()
                            .clear_bit()
                            .spe()
                            .set_bit()
                    });

                    // XXX: what about cr2?

                    Spi { spi, pins }
                }

                pub fn free(self) -> ($SPIX, PINS) {
                    (self.spi, self.pins)
                }
            }

            impl SpiExt<$SPIX> for $SPIX {
                fn spi<PINS, T>(self, pins: PINS, mode: Mode, freq: T, rcc: &mut Rcc) -> Spi<$SPIX, PINS>
                where
                    PINS: Pins<$SPIX>,
                    T: Into<Hertz>
                    {
                        Spi::$spiX(self, pins, mode, freq, rcc)
                    }
            }

            impl<PINS> hal::spi::FullDuplex<u8> for Spi<$SPIX, PINS> {
                type Error = Error;

                fn read(&mut self) -> nb::Result<u8, Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().bit_is_set() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().bit_is_set() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.rxne().bit_is_set() {
                        // NOTE(read_volatile) read only 1 byte (the svd2rust API only allows
                        // reading a half-word)
                        return Ok(unsafe {
                            ptr::read_volatile(&self.spi.dr as *const _ as *const u8)
                        });
                    } else {
                        nb::Error::WouldBlock
                    })
                }

                fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
                    let sr = self.spi.sr.read();

                    Err(if sr.ovr().bit_is_set() {
                        nb::Error::Other(Error::Overrun)
                    } else if sr.modf().bit_is_set() {
                        nb::Error::Other(Error::ModeFault)
                    } else if sr.crcerr().bit_is_set() {
                        nb::Error::Other(Error::Crc)
                    } else if sr.txe().bit_is_set() {
                        // NOTE(write_volatile) see note above
                        unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut u8, byte) }
                        return Ok(());
                    } else {
                        nb::Error::WouldBlock
                    })
                }

            }

            impl<PINS> crate::hal::blocking::spi::transfer::Default<u8> for Spi<$SPIX, PINS> {}

            impl<PINS> crate::hal::blocking::spi::write::Default<u8> for Spi<$SPIX, PINS> {}
        )+
    }
}

spi! {
    SPI1: (spi1, apb2enr, spi1en, pclk1),
}

spi! {
    SPI2: (spi2, apb1enr1, spi2en, pclk1),
}
