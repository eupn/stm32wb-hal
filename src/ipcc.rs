use crate::rcc::Rcc;
use crate::pac::{self, IPCC};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum IpccChannel {
    Channel1 = 0x00000001,
    Channel2 = 0x00000002,
    Channel3 = 0x00000004,
    Channel4 = 0x00000008,
    Channel5 = 0x00000010,
    Channel6 = 0x00000020,
}

impl IpccChannel {
    pub fn iterator() -> IpccChannelIterator {
        IpccChannelIterator { channel_number: 0 }
    }
}

pub struct IpccChannelIterator {
    channel_number: u8,
}

impl Iterator for IpccChannelIterator {
    type Item = IpccChannel;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = match self.channel_number {
            0 => Some(IpccChannel::Channel1),
            1 => Some(IpccChannel::Channel2),
            2 => Some(IpccChannel::Channel3),
            3 => Some(IpccChannel::Channel4),
            4 => Some(IpccChannel::Channel5),
            5 => Some(IpccChannel::Channel6),

            _ => None,
        };

        self.channel_number += 1;

        ch
    }
}

pub struct Ipcc {
    pub rb: IPCC,
}

impl Ipcc {
    /// Enables clocking of IPCC and unmasks two associated interrupts: `IPCC_C1_RX` and `IPCC_C1_TX`.
    pub fn init(&mut self, rcc: &mut Rcc) {
        rcc.set_ipcc(true);

        // Enable IPCC interrupts
        self.rb
            .c1cr
            .modify(|_, w| w.rxoie().set_bit().txfie().set_bit());
        unsafe {
            cortex_m::peripheral::NVIC::unmask(pac::interrupt::IPCC_C1_RX_IT);
            cortex_m::peripheral::NVIC::unmask(pac::interrupt::IPCC_C1_TX_IT);
        }
    }

    /// Resets IPCC to the default state.
    pub fn reset(&mut self) {
        for channel in IpccChannel::iterator() {
            self.c1_clear_flag_channel(channel);
            self.c2_clear_flag_channel(channel);

            self.c1_set_rx_channel(channel, false);
            self.c2_set_rx_channel(channel, false);

            self.c1_set_tx_channel(channel, false);
            self.c2_set_tx_channel(channel, false);
        }
    }

    pub fn c1_set_rx_channel(&mut self, channel: IpccChannel, enabled: bool) {
        // If bit is set to 1 then interrupt is disabled
        self.rb.c1mr.modify(|_, w| match channel {
            IpccChannel::Channel1 => w.ch1om().bit(!enabled),
            IpccChannel::Channel2 => w.ch2om().bit(!enabled),
            IpccChannel::Channel3 => w.ch3om().bit(!enabled),
            IpccChannel::Channel4 => w.ch4om().bit(!enabled),
            IpccChannel::Channel5 => w.ch5om().bit(!enabled),
            IpccChannel::Channel6 => w.ch6om().bit(!enabled),
        });
    }

    pub fn c1_get_rx_channel(&self, channel: IpccChannel) -> bool {
        !match channel {
            IpccChannel::Channel1 => self.rb.c1mr.read().ch1om().bit(),
            IpccChannel::Channel2 => self.rb.c1mr.read().ch2om().bit(),
            IpccChannel::Channel3 => self.rb.c1mr.read().ch3om().bit(),
            IpccChannel::Channel4 => self.rb.c1mr.read().ch4om().bit(),
            IpccChannel::Channel5 => self.rb.c1mr.read().ch5om().bit(),
            IpccChannel::Channel6 => self.rb.c1mr.read().ch6om().bit(),
        }
    }

    pub fn c2_set_rx_channel(&mut self, channel: IpccChannel, enabled: bool) {
        // If bit is set to 1 then interrupt is disabled
        self.rb.c2mr.modify(|_, w| match channel {
            IpccChannel::Channel1 => w.ch1om().bit(!enabled),
            IpccChannel::Channel2 => w.ch2om().bit(!enabled),
            IpccChannel::Channel3 => w.ch3om().bit(!enabled),
            IpccChannel::Channel4 => w.ch4om().bit(!enabled),
            IpccChannel::Channel5 => w.ch5om().bit(!enabled),
            IpccChannel::Channel6 => w.ch6om().bit(!enabled),
        });
    }

    pub fn c1_set_tx_channel(&mut self, channel: IpccChannel, enabled: bool) {
        // If bit is set to 1 then interrupt is disabled
        self.rb.c1mr.modify(|_, w| match channel {
            IpccChannel::Channel1 => w.ch1fm().bit(!enabled),
            IpccChannel::Channel2 => w.ch2fm().bit(!enabled),
            IpccChannel::Channel3 => w.ch3fm().bit(!enabled),
            IpccChannel::Channel4 => w.ch4fm().bit(!enabled),
            IpccChannel::Channel5 => w.ch5fm().bit(!enabled),
            IpccChannel::Channel6 => w.ch6fm().bit(!enabled),
        });
    }

    pub fn c1_get_tx_channel(&self, channel: IpccChannel) -> bool {
        !match channel {
            IpccChannel::Channel1 => self.rb.c1mr.read().ch1fm().bit(),
            IpccChannel::Channel2 => self.rb.c1mr.read().ch2fm().bit(),
            IpccChannel::Channel3 => self.rb.c1mr.read().ch3fm().bit(),
            IpccChannel::Channel4 => self.rb.c1mr.read().ch4fm().bit(),
            IpccChannel::Channel5 => self.rb.c1mr.read().ch5fm().bit(),
            IpccChannel::Channel6 => self.rb.c1mr.read().ch6fm().bit(),
        }
    }

    pub fn c2_set_tx_channel(&mut self, channel: IpccChannel, enabled: bool) {
        // If bit is set to 1 then interrupt is disabled
        self.rb.c2mr.modify(|_, w| match channel {
            IpccChannel::Channel1 => w.ch1fm().bit(!enabled),
            IpccChannel::Channel2 => w.ch2fm().bit(!enabled),
            IpccChannel::Channel3 => w.ch3fm().bit(!enabled),
            IpccChannel::Channel4 => w.ch4fm().bit(!enabled),
            IpccChannel::Channel5 => w.ch5fm().bit(!enabled),
            IpccChannel::Channel6 => w.ch6fm().bit(!enabled),
        });
    }

    /// Clears IPCC receive channel status for CPU1.
    pub fn c1_clear_flag_channel(&mut self, channel: IpccChannel) {
        match channel {
            IpccChannel::Channel1 => self.rb.c1scr.write(|w| w.ch1c().set_bit()),
            IpccChannel::Channel2 => self.rb.c1scr.write(|w| w.ch2c().set_bit()),
            IpccChannel::Channel3 => self.rb.c1scr.write(|w| w.ch3c().set_bit()),
            IpccChannel::Channel4 => self.rb.c1scr.write(|w| w.ch4c().set_bit()),
            IpccChannel::Channel5 => self.rb.c1scr.write(|w| w.ch5c().set_bit()),
            IpccChannel::Channel6 => self.rb.c1scr.write(|w| w.ch6c().set_bit()),
        }
    }

    /// Clears IPCC receive channel status for CPU2.
    pub fn c2_clear_flag_channel(&mut self, channel: IpccChannel) {
        match channel {
            IpccChannel::Channel1 => self.rb.c2scr.write(|w| w.ch1c().set_bit()),
            IpccChannel::Channel2 => self.rb.c2scr.write(|w| w.ch2c().set_bit()),
            IpccChannel::Channel3 => self.rb.c2scr.write(|w| w.ch3c().set_bit()),
            IpccChannel::Channel4 => self.rb.c2scr.write(|w| w.ch4c().set_bit()),
            IpccChannel::Channel5 => self.rb.c2scr.write(|w| w.ch5c().set_bit()),
            IpccChannel::Channel6 => self.rb.c2scr.write(|w| w.ch6c().set_bit()),
        }
    }

    /// Sets IPCC transmit channel status for CPU1.
    pub fn c1_set_flag_channel(&mut self, channel: IpccChannel) {
        match channel {
            IpccChannel::Channel1 => self.rb.c1scr.write(|w| w.ch1s().set_bit()),
            IpccChannel::Channel2 => self.rb.c1scr.write(|w| w.ch2s().set_bit()),
            IpccChannel::Channel3 => self.rb.c1scr.write(|w| w.ch3s().set_bit()),
            IpccChannel::Channel4 => self.rb.c1scr.write(|w| w.ch4s().set_bit()),
            IpccChannel::Channel5 => self.rb.c1scr.write(|w| w.ch5s().set_bit()),
            IpccChannel::Channel6 => self.rb.c1scr.write(|w| w.ch6s().set_bit()),
        }
    }

    /// Sets IPCC transmit channel status for CPU2.
    pub fn c2_set_flag_channel(&mut self, channel: IpccChannel) {
        match channel {
            IpccChannel::Channel1 => self.rb.c2scr.write(|w| w.ch1s().set_bit()),
            IpccChannel::Channel2 => self.rb.c2scr.write(|w| w.ch2s().set_bit()),
            IpccChannel::Channel3 => self.rb.c2scr.write(|w| w.ch3s().set_bit()),
            IpccChannel::Channel4 => self.rb.c2scr.write(|w| w.ch4s().set_bit()),
            IpccChannel::Channel5 => self.rb.c2scr.write(|w| w.ch5s().set_bit()),
            IpccChannel::Channel6 => self.rb.c2scr.write(|w| w.ch6s().set_bit()),
        }
    }

    pub fn c1_is_active_flag(&self, channel: IpccChannel) -> bool {
        match channel {
            IpccChannel::Channel1 => self.rb.c1toc2sr.read().ch1f().bit(),
            IpccChannel::Channel2 => self.rb.c1toc2sr.read().ch2f().bit(),
            IpccChannel::Channel3 => self.rb.c1toc2sr.read().ch3f().bit(),
            IpccChannel::Channel4 => self.rb.c1toc2sr.read().ch4f().bit(),
            IpccChannel::Channel5 => self.rb.c1toc2sr.read().ch5f().bit(),
            IpccChannel::Channel6 => self.rb.c1toc2sr.read().ch6f().bit(),
        }
    }

    pub fn c2_is_active_flag(&self, channel: IpccChannel) -> bool {
        match channel {
            IpccChannel::Channel1 => self.rb.c2toc1sr.read().ch1f().bit(),
            IpccChannel::Channel2 => self.rb.c2toc1sr.read().ch2f().bit(),
            IpccChannel::Channel3 => self.rb.c2toc1sr.read().ch3f().bit(),
            IpccChannel::Channel4 => self.rb.c2toc1sr.read().ch4f().bit(),
            IpccChannel::Channel5 => self.rb.c2toc1sr.read().ch5f().bit(),
            IpccChannel::Channel6 => self.rb.c2toc1sr.read().ch6f().bit(),
        }
    }

    pub fn is_tx_pending(&self, channel: IpccChannel) -> bool {
        !self.c1_is_active_flag(channel) && self.c1_get_tx_channel(channel)
    }

    pub fn is_rx_pending(&self, channel: IpccChannel) -> bool {
        self.c2_is_active_flag(channel) && self.c1_get_rx_channel(channel)
    }
}

/// Extension trait that constrains the `IPCC` peripheral
pub trait IpccExt {
    /// Constrains the `IPCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Ipcc;
}

impl IpccExt for IPCC {
    fn constrain(self) -> Ipcc {
        Ipcc { rb: self }
    }
}
