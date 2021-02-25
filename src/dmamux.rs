use crate::pac::DMAMUX1;

/// Extension trait to split a DMA peripheral into independent channels
pub trait DmaMuxExt {
    /// The type to split the DMA into
    type Channels;

    /// Split the DMA into independent channels
    fn split(self) -> Self::Channels;
}

#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum DmaMuxIndex {
    dmamux_req_gen0 = 1,
    dmamux_req_gen1 = 2,
    dmamux_req_gen2 = 3,
    dmamux_req_gen3 = 4,

    ADC1 = 5,

    SPI1_RX = 6,
    SPI1_TX = 7,

    I2C1_RX = 10,
    I2C1_TX = 11,

    I2C3_RX = 12,
    I2C3_TX = 13,
    // TODO: add more peripherals
}

impl DmaMuxIndex {
    pub fn val(self) -> u8 {
        self as u8
    }
}

#[allow(non_camel_case_types)]
pub enum DmaMuxTriggerSync {
    EXTI_LINE0 = 0,
    EXTI_LINE1 = 1,
    EXTI_LINE2 = 2,
    EXTI_LINE3 = 3,
    EXTI_LINE4 = 4,
    EXTI_LINE5 = 5,
    EXTI_LINE6 = 6,
    EXTI_LINE7 = 7,
    EXTI_LINE8 = 8,
    EXTI_LINE9 = 9,
    EXTI_LINE10 = 10,
    EXTI_LINE11 = 11,
    EXTI_LINE12 = 12,
    EXTI_LINE13 = 13,
    EXTI_LINE14 = 14,
    EXTI_LINE15 = 15,
    dmamux_evt0 = 16,
    dmamux_evt1 = 17,

    LPTIM1_OUT = 18,
    LPTIM2_OUT = 19,
}

impl DmaMuxTriggerSync {
    pub fn val(self) -> u8 {
        self as u8
    }
}

pub trait DmaMuxChannel {
    fn select_peripheral(&mut self, index: DmaMuxIndex);
}

macro_rules! dma_mux {
    (
        channels: {
            $( $Ci:ident: ($chi:ident, $cr:ident), )+
        },
    ) => {

        /// DMAMUX channels
        pub struct Channels {
            $( pub $chi: $Ci, )+
        }

        $(
            /// Singleton that represents a DMAMUX channel
            pub struct $Ci {
                _0: (),
            }

            impl DmaMuxChannel for $Ci {
                fn select_peripheral(&mut self, index: DmaMuxIndex) {
                    let reg = unsafe { &(*DMAMUX1::ptr()).$cr };
                    reg.write( |w| unsafe {
                        w
                        .dmareq_id().bits(index.val())
                        .ege().set_bit()
                    });
                }
            }
        )+

    }
}

dma_mux!(
    channels: {
        C0: (ch0, c0cr),
        C1: (ch1, c1cr),
        C2: (ch2, c2cr),
        C3: (ch3, c3cr),
        C4: (ch4, c4cr),
        C5: (ch5, c5cr),
        C6: (ch6, c6cr),
    },
);

impl DmaMuxExt for DMAMUX1 {
    type Channels = Channels;

    fn split(self) -> Self::Channels {
        Channels {
            ch0: C0 { _0: () },
            ch1: C1 { _0: () },
            ch2: C2 { _0: () },
            ch3: C3 { _0: () },
            ch4: C4 { _0: () },
            ch5: C5 { _0: () },
            ch6: C6 { _0: () },
        }
    }
}
