//! # Direct Memory Access
#![allow(dead_code)]

use core::{
    marker::PhantomData,
    sync::atomic::{compiler_fence, Ordering},
};
use embedded_dma::{StaticReadBuffer, StaticWriteBuffer};

use crate::rcc::Rcc;
use crate::pac::DMAMUX1;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Overrun,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Half {
    First,
    Second,
}

pub struct CircBuffer<BUFFER, PAYLOAD>
    where
        BUFFER: 'static,
{
    buffer: &'static mut [BUFFER; 2],
    payload: PAYLOAD,
    readable_half: Half,
}

impl<BUFFER, PAYLOAD> CircBuffer<BUFFER, PAYLOAD>
    where
        &'static mut [BUFFER; 2]: StaticWriteBuffer,
        BUFFER: 'static,
{
    pub(crate) fn new(buf: &'static mut [BUFFER; 2], payload: PAYLOAD) -> Self {
        CircBuffer {
            buffer: buf,
            payload,
            readable_half: Half::Second,
        }
    }
}

pub trait DmaExt {
    type Channels;

    fn split(self, rcc: &mut Rcc, dmamux: DMAMUX1) -> Self::Channels;
}

pub trait TransferPayload {
    fn start(&mut self);
    fn stop(&mut self);
}

pub struct Transfer<MODE, BUFFER, PAYLOAD>
    where
        PAYLOAD: TransferPayload,
{
    _mode: PhantomData<MODE>,
    buffer: BUFFER,
    payload: PAYLOAD,
}

impl<BUFFER, PAYLOAD> Transfer<R, BUFFER, PAYLOAD>
    where
        PAYLOAD: TransferPayload,
{
    pub(crate) fn r(buffer: BUFFER, payload: PAYLOAD) -> Self {
        Transfer {
            _mode: PhantomData,
            buffer,
            payload,
        }
    }
}

impl<BUFFER, PAYLOAD> Transfer<W, BUFFER, PAYLOAD>
    where
        PAYLOAD: TransferPayload,
{
    pub(crate) fn w(buffer: BUFFER, payload: PAYLOAD) -> Self {
        Transfer {
            _mode: PhantomData,
            buffer,
            payload,
        }
    }
}

impl<MODE, BUFFER, PAYLOAD> Drop for Transfer<MODE, BUFFER, PAYLOAD>
    where
        PAYLOAD: TransferPayload,
{
    fn drop(&mut self) {
        self.payload.stop();
        compiler_fence(Ordering::SeqCst);
    }
}

/// Channel priority level
pub enum Priority {
    /// Low
    Low = 0b00,
    /// Medium
    Medium = 0b01,
    /// High
    High = 0b10,
    /// Very high
    VeryHigh = 0b11,
}

impl From<Priority> for u8 {
    fn from(prio: Priority) -> Self {
        match prio {
            Priority::Low => 0b00,
            Priority::Medium => 0b01,
            Priority::High => 0b10,
            Priority::VeryHigh => 0b11,
        }
    }
}

/// DMA transfer direction
pub enum Direction {
    /// From memory to peripheral
    FromMemory,
    /// From peripheral to memory
    FromPeripheral,
}

impl From<Direction> for bool {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::FromMemory => true,
            Direction::FromPeripheral => false,
        }
    }
}

#[doc = "Peripheral size"]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum WordSize {
    #[doc = "0: 8-bit size"]
    BITS8 = 0,
    #[doc = "1: 16-bit size"]
    BITS16 = 1,
    #[doc = "2: 32-bit size"]
    BITS32 = 2,
}
impl From<WordSize> for u8 {
    #[inline(always)]
    fn from(variant: WordSize) -> Self {
        variant as _
    }
}

/// DMA events
pub enum Event {
    /// First half of a transfer is done
    HalfTransfer,
    /// Transfer is complete
    TransferComplete,
    /// A transfer error occurred
    TransferError,
    /// Any of the above events occurred
    Any,
}

/// Read transfer
pub struct R;

/// Write transfer
pub struct W;

macro_rules! dma {
    ($($DMAX:ident: ($dmaX:ident, $dmaMod:ident, $dmaen:ident, $dmarst: ident, {
        $($CX:ident: (
            $chX:ident,
            $cpar:ident,
            $cmar:ident,
            $ccr:ident,
            $cndtr:ident,
            $htifX:ident,
            $tcifX:ident,
            $chtifX:ident,
            $ctcifX:ident,
            $cgifX:ident,
            $muxCi:ident
        ),)+
    }),)+) => {
        $(
            pub mod $dmaMod {
                use core::{sync::atomic::{self, Ordering}, ptr, mem};

                use crate::pac::{$DMAX, $dmaX};
                use crate::pac::DMAMUX1;

                use crate::rcc::Rcc;
                use crate::dma::{CircBuffer, DmaExt, Error, Half, Transfer, W, RxDma, TxDma, TransferPayload};
                use crate::dmamux::DmaMuxExt;

                /// DMA channels
                pub struct Channels {
                    $( pub $chX: $CX, )+
                }

                impl Channels {
                    /// Reset the control registers of all channels.
                    /// This stops any ongoing transfers.
                    fn reset(&mut self) {
                        $( self.$chX.reset(); )+
                    }
                }

                $(
                    /// A singleton that represents a single DMAx channel (channel X in this case)
                    ///
                    /// This singleton has exclusive access to the registers of the DMAx channel X
                    pub struct $CX {
                        mux: crate::dmamux::$muxCi,
                    }

                    impl $CX {
                        /// Reset the control registers of this channel.
                        /// This stops any ongoing transfers.
                        fn reset(&mut self) {
                            self.rb().$ccr.reset();
                            self.rb().$cndtr.reset();
                            self.rb().$cpar.reset();
                            self.rb().$cmar.reset();
                        }

                        /// Enables or disables memory-to-memory transfers
                        pub fn set_mem2mem(&mut self, enable: bool) {
                            if enable {
                                self.rb().$ccr.modify(|_, w| w.mem2mem().set_bit());
                            } else {
                                self.rb().$ccr.modify(|_, w| w.mem2mem().clear_bit());
                            }
                        }

                        /// Associated peripheral `address`
                        ///
                        /// `inc` indicates whether the address will be incremented after every byte transfer
                        pub fn set_peripheral_address(&mut self, address: u32, inc: bool) {
                            self.rb().$cpar.write(|w| unsafe { w.pa().bits(address) });
                            self.rb().$ccr.modify(|_, w| w.pinc().bit(inc) );
                        }

                        /// `address` where from/to data will be read/write
                        ///
                        /// `inc` indicates whether the address will be incremented after every byte transfer
                        pub fn set_memory_address(&mut self, address: u32, inc: bool) {
                            self.rb().$cmar.write(|w| unsafe { w.ma().bits(address) } );
                            self.rb().$ccr.modify(|_, w| w.minc().bit(inc) );
                        }

                        /// Number of bytes to transfer
                        pub fn set_transfer_length(&mut self, len: usize) {
                            self.rb().$cndtr.write(|w| unsafe { w.ndt().bits(cast::u16(len).unwrap()) });
                        }

                        /// Set the word size.
                        pub fn set_word_size(&mut self, wsize: super::WordSize) {
                            self.rb().$ccr.modify(|_, w| unsafe {
                                w.psize().bits(wsize as u8);
                                w.msize().bits(wsize as u8)
                            });
                        }

                        /// Set the priority level of this channel
                        pub fn set_priority_level(&mut self, priority: super::Priority) {
                            let pl = priority.into();
                            self.rb().$ccr.modify(|_, w| unsafe { w.pl().bits(pl) });
                        }

                        /// Set the transfer direction
                        pub fn set_direction(&mut self, direction: super::Direction) {
                            let dir = direction.into();
                            self.rb().$ccr.modify(|_, w| w.dir().bit(dir));
                        }

                        /// Set the circular mode of this channel
                        pub fn set_circular_mode(&mut self, circular: bool) {
                            self.rb().$ccr.modify(|_, w| w.circ().bit(circular));
                        }

                        /// Starts the DMA transfer
                        pub fn start(&mut self) {
                            self.rb().$ccr.modify(|_, w| w.en().set_bit() );
                        }

                        /// Stops the DMA transfer
                        pub fn stop(&mut self) {
                            self.rb().ifcr.write(|w| w.$cgifX().set_bit());
                            self.rb().$ccr.modify(|_, w| w.en().clear_bit() );
                        }

                        /// Returns `true` if there's a transfer in progress
                        pub fn in_progress(&self) -> bool {
                            self.isr().$tcifX().bit_is_clear()
                        }

                        #[inline]
                        pub(crate) fn get_ndtr(&self) -> u32 {
                            // NOTE(unsafe) atomic read with no side effects
                            unsafe { (*$DMAX::ptr()).$cndtr.read().bits() }
                        }

                        #[inline]
                        pub fn isr(&self) -> $dmaX::isr::R {
                            // NOTE(unsafe) atomic read with no side effects
                            unsafe { (*$DMAX::ptr()).isr.read() }
                        }

                        pub fn mux(&mut self) -> &mut dyn crate::dmamux::DmaMuxChannel {
                            &mut self.mux
                        }

                        pub fn select_peripheral(&mut self, index: crate::dmamux::DmaMuxIndex) {
                            self.mux().select_peripheral(index);
                        }
                    }

                    impl $CX {
                        /// Enable the interrupt for the given event
                        pub fn listen(&mut self, event: super::Event) {
                            use super::Event::*;
                            match event {
                                HalfTransfer => self.rb().$ccr.modify(|_, w| w.htie().set_bit()),
                                TransferComplete => self.rb().$ccr.modify(|_, w| w.tcie().set_bit()),
                                TransferError => self.rb().$ccr.modify(|_, w| w.teie().set_bit()),
                                Any => self.rb().$ccr.modify(|_, w| {
                                    w.htie().set_bit();
                                    w.tcie().set_bit();
                                    w.teie().set_bit()
                                }),
                            }
                        }

                        /// Disable the interrupt for the given event
                        pub fn unlisten(&mut self, event: super::Event) {
                            use super::Event::*;
                            match event {
                                HalfTransfer => self.rb().$ccr.modify(|_, w| w.htie().clear_bit()),
                                TransferComplete => self.rb().$ccr.modify(|_, w| w.tcie().clear_bit()),
                                TransferError => self.rb().$ccr.modify(|_, w| w.teie().clear_bit()),
                                Any => self.rb().$ccr.modify(|_, w| {
                                    w.htie().clear_bit();
                                    w.tcie().clear_bit();
                                    w.teie().clear_bit()
                                }),
                            }
                        }

                        pub fn rb(&mut self) -> &crate::pac::$dmaX::RegisterBlock {
                            unsafe { &(*$DMAX::ptr()) }
                        }
                    }

                    impl<B, PAYLOAD> CircBuffer<B, RxDma<PAYLOAD, $CX>>
                    where
                        RxDma<PAYLOAD, $CX>: TransferPayload,
                    {
                        /// Peeks into the readable half of the buffer
                        pub fn peek<R, F>(&mut self, f: F) -> Result<R, Error>
                            where
                            F: FnOnce(&B, Half) -> R,
                        {
                            let half_being_read = self.readable_half()?;

                            let buf = match half_being_read {
                                Half::First => &self.buffer[0],
                                Half::Second => &self.buffer[1],
                            };

                            // XXX does this need a compiler barrier?
                            let ret = f(buf, half_being_read);


                            let isr = self.payload.channel.rb().isr.read();
                            let first_half_is_done = isr.$htifX().bit_is_set();
                            let second_half_is_done = isr.$tcifX().bit_is_set();

                            if (half_being_read == Half::First && second_half_is_done) ||
                                (half_being_read == Half::Second && first_half_is_done) {
                                Err(Error::Overrun)
                            } else {
                                Ok(ret)
                            }
                        }

                        /// Returns the `Half` of the buffer that can be read
                        pub fn readable_half(&mut self) -> Result<Half, Error> {
                            let isr = self.payload.channel.rb().isr.read();
                            let first_half_is_done = isr.$htifX().bit_is_set();
                            let second_half_is_done = isr.$tcifX().bit_is_set();

                            if first_half_is_done && second_half_is_done {
                                return Err(Error::Overrun);
                            }

                            let last_read_half = self.readable_half;

                            Ok(match last_read_half {
                                Half::First => {
                                    if second_half_is_done {
                                        self.payload.channel.rb().ifcr.write(|w| w.$ctcifX().set_bit());

                                        self.readable_half = Half::Second;
                                        Half::Second
                                    } else {
                                        last_read_half
                                    }
                                }
                                Half::Second => {
                                    if first_half_is_done {
                                        self.payload.channel.rb().ifcr.write(|w| w.$chtifX().set_bit());

                                        self.readable_half = Half::First;
                                        Half::First
                                    } else {
                                        last_read_half
                                    }
                                }
                            })
                        }

                        /// Stops the transfer and returns the underlying buffer and RxDma
                        pub fn stop(mut self) -> (&'static mut [B; 2], RxDma<PAYLOAD, $CX>) {
                            self.payload.stop();

                            (self.buffer, self.payload)
                        }
                    }

                    impl<BUFFER, PAYLOAD, MODE> Transfer<MODE, BUFFER, RxDma<PAYLOAD, $CX>>
                    where
                        RxDma<PAYLOAD, $CX>: TransferPayload,
                    {
                        pub fn is_done(&self) -> bool {
                            !self.payload.channel.in_progress()
                        }

                        pub fn wait(self) -> (BUFFER, RxDma<PAYLOAD, $CX>) {
                            while !self.is_done() {}
                            self.destroy()
                        }

                        pub fn destroy(mut self) -> (BUFFER, RxDma<PAYLOAD, $CX>) {
                            atomic::compiler_fence(Ordering::Acquire);
                            self.payload.stop();

                            // we need a read here to make the Acquire fence effective
                            // we do *not* need this if `dma.stop` does a RMW operation
                            unsafe { ptr::read_volatile(&0); }

                            // we need a fence here for the same reason we need one in `Transfer.wait`
                            atomic::compiler_fence(Ordering::Acquire);

                            // `Transfer` needs to have a `Drop` implementation, because we accept
                            // managed buffers that can free their memory on drop. Because of that
                            // we can't move out of the `Transfer`'s fields, so we use `ptr::read`
                            // and `mem::forget`.
                            //
                            // NOTE(unsafe) There is no panic branch between getting the resources
                            // and forgetting `self`.
                            unsafe {
                                let buffer = ptr::read(&self.buffer);
                                let payload = ptr::read(&self.payload);
                                mem::forget(self);
                                (buffer, payload)
                            }
                        }
                    }

                    impl<BUFFER, PAYLOAD, MODE> Transfer<MODE, BUFFER, TxDma<PAYLOAD, $CX>>
                    where
                        TxDma<PAYLOAD, $CX>: TransferPayload,
                    {
                        pub fn is_done(&self) -> bool {
                            !self.payload.channel.in_progress()
                        }

                        pub fn wait(self) -> (BUFFER, TxDma<PAYLOAD, $CX>) {
                            while !self.is_done() {}
                            self.destroy()
                        }

                        pub fn destroy(mut self) -> (BUFFER, TxDma<PAYLOAD, $CX>) {
                            atomic::compiler_fence(Ordering::Acquire);

                            self.payload.stop();

                            // we need a read here to make the Acquire fence effective
                            // we do *not* need this if `dma.stop` does a RMW operation
                            unsafe { ptr::read_volatile(&0); }

                            // we need a fence here for the same reason we need one in `Transfer.wait`
                            atomic::compiler_fence(Ordering::Acquire);

                            // `Transfer` needs to have a `Drop` implementation, because we accept
                            // managed buffers that can free their memory on drop. Because of that
                            // we can't move out of the `Transfer`'s fields, so we use `ptr::read`
                            // and `mem::forget`.
                            //
                            // NOTE(unsafe) There is no panic branch between getting the resources
                            // and forgetting `self`.
                            unsafe {
                                let buffer = ptr::read(&self.buffer);
                                let payload = ptr::read(&self.payload);
                                mem::forget(self);
                                (buffer, payload)
                            }
                        }
                    }

                    impl<BUFFER, PAYLOAD> Transfer<W, BUFFER, RxDma<PAYLOAD, $CX>>
                    where
                        RxDma<PAYLOAD, $CX>: TransferPayload,
                    {
                        pub fn peek<T>(&self) -> &[T]
                        where
                            BUFFER: AsRef<[T]>,
                        {
                            let pending = self.payload.channel.get_ndtr() as usize;

                            let slice = self.buffer.as_ref();
                            let capacity = slice.len();

                            &slice[..(capacity - pending)]
                        }
                    }
                )+

                impl DmaExt for $DMAX {
                    type Channels = Channels;

                    fn split(self, rcc: &mut Rcc, dmamux: DMAMUX1) -> Channels {
                        let muxchannels = dmamux.split();
                        // enable DMAMUX & DMA clock
                        rcc.rb.ahb1enr.modify(|_, w| w.dmamuxen().set_bit());
                        rcc.rb.ahb1enr.modify(|_, w| w.$dmaen().set_bit());

                        let mut channels = Channels {
                            ch1: C1 {
                                mux: muxchannels.ch0,
                            },
                            ch2: C2 {
                                mux: muxchannels.ch1,
                            },
                            ch3: C3 {
                                mux: muxchannels.ch2,
                            },
                            ch4: C4 {
                                mux: muxchannels.ch3,
                            },
                            ch5: C5 {
                                mux: muxchannels.ch4,
                            },
                            ch6: C6 {
                                mux: muxchannels.ch5,
                            },
                            ch7: C7 {
                                mux: muxchannels.ch6,
                            },
                        };
                        channels.reset();
                        channels
                    }
                }
            }
        )+
    }
}

dma! {
    DMA1: (dma1, dma1impl, dma1en, dma1rst, {
        C1: (
            ch1,
            cpar1, cmar1, ccr1, cndtr1,
            htif1, tcif1,
            chtif1, ctcif1, cgif1,
            C0
        ),
        C2: (
            ch2,
            cpar2, cmar2, ccr2, cndtr2,
            htif2, tcif2,
            chtif2, ctcif2, cgif2,
            C1
        ),
        C3: (
            ch3,
            cpar3, cmar3, ccr3, cndtr3,
            htif3, tcif3,
            chtif3, ctcif3, cgif3,
            C2
        ),
        C4: (
            ch4,
            cpar3, cmar3, ccr3, cndtr3,
            htif4, tcif4,
            chtif4, ctcif4, cgif4,
            C3
        ),
        C5: (
            ch5,
            cpar5, cmar5, ccr5, cndtr5,
            htif5, tcif5,
            chtif5, ctcif5, cgif5,
            C4
        ),
        C6: (
            ch6,
            cpar6, cmar6, ccr6, cndtr6,
            htif6, tcif6,
            chtif6, ctcif6, cgif6,
            C5
        ),
        C7: (
            ch7,
            cpar7, cmar7, ccr7, cndtr7,
            htif7, tcif7,
            chtif7, ctcif7, cgif7,
            C6
        ),
    }),

    DMA2: (dma2, dma2impl, dma2en, dma2rst, {
        C1: (
            ch1,
            cpar1, cmar1, ccr1, cndtr1,
            htif1, tcif1,
            chtif1, ctcif1, cgif1,
            C0
        ),
        C2: (
            ch2,
            cpar2, cmar2, ccr2, cndtr2,
            htif2, tcif2,
            chtif2, ctcif2, cgif2,
            C1
        ),
        C3: (
            ch3,
            cpar3, cmar3, ccr3, cndtr3,
            htif3, tcif3,
            chtif3, ctcif3, cgif3,
            C2
        ),
        C4: (
            ch4,
            cpar4, cmar4, ccr4, cndtr4,
            htif4, tcif4,
            chtif4, ctcif4, cgif4,
            C3
        ),
        C5: (
            ch5,
            cpar5, cmar5, ccr5, cndtr5,
            htif5, tcif5,
            chtif5, ctcif5, cgif5,
            C4
        ),
        C6: (
            ch6,
            cpar6, cmar6, ccr6, cndtr6,
            htif6, tcif6,
            chtif6, ctcif6, cgif6,
            C5
        ),
        C7: (
            ch7,
            cpar7, cmar7, ccr7, cndtr7,
            htif7, tcif7,
            chtif7, ctcif7, cgif7,
            C6
        ),
    }),
}

/// DMA Receiver
pub struct RxDma<PAYLOAD, RXCH> {
    pub(crate) payload: PAYLOAD,
    pub channel: RXCH,
}

/// DMA Transmitter
pub struct TxDma<PAYLOAD, TXCH> {
    pub(crate) payload: PAYLOAD,
    pub channel: TXCH,
}

/// DMA Receiver/Transmitter
pub struct RxTxDma<PAYLOAD, RXCH, TXCH> {
    pub(crate) payload: PAYLOAD,
    pub rxchannel: RXCH,
    pub txchannel: TXCH,
}

pub trait Receive {
    type RxChannel;
    type TransmittedWord;
}

pub trait Transmit {
    type TxChannel;
    type ReceivedWord;
}

/// Trait for circular DMA readings from peripheral to memory.
pub trait CircReadDma<B, RS>: Receive
    where
        &'static mut [B; 2]: StaticWriteBuffer<Word = RS>,
        B: 'static,
        Self: core::marker::Sized,
{
    fn circ_read(self, buffer: &'static mut [B; 2]) -> CircBuffer<B, Self>;
}

/// Trait for DMA readings from peripheral to memory.
pub trait ReadDma<B, RS>: Receive
    where
        B: StaticWriteBuffer<Word = RS>,
        Self: core::marker::Sized + TransferPayload,
{
    fn read(self, buffer: B) -> Transfer<W, B, Self>;
}

/// Trait for DMA writing from memory to peripheral.
pub trait WriteDma<B, TS>: Transmit
    where
        B: StaticReadBuffer<Word = TS>,
        Self: core::marker::Sized + TransferPayload,
{
    fn write(self, buffer: B) -> Transfer<R, B, Self>;
}
