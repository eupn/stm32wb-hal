//! Flash memory module

use crate::stm32::{flash, FLASH};
use crate::traits::flash as flash_trait;
use core::convert::TryInto;
use core::{mem, ops::Drop, ptr};
pub use flash_trait::{Error, FlashPage, Read, WriteErase};

/// Extension trait to constrain the FLASH peripheral
pub trait FlashExt {
    /// Constrains the FLASH peripheral to play nicely with the other abstractions
    fn constrain(self) -> Parts;
}

impl FlashExt for FLASH {
    fn constrain(self) -> Parts {
        Parts {
            acr: ACR {},
            keyr: KEYR {},
            optkeyr: OPTKEYR {},
            sr: SR {},
            c2sr: C2SR {},
            cr: CR {},
            eccr: ECCR {},
            pcrop1asr: PCROP1ASR {},
            pcrop1aer: PCROP1AER {},
            wrp1ar: WRP1AR {},
            wrp1br: WRP1BR {},
        }
    }
}

/// Constrained FLASH peripheral
pub struct Parts {
    /// Opaque ACR register
    pub acr: ACR,
    /// Opaque KEYR register
    pub keyr: KEYR,
    /// Opaque OPTKEYR register
    pub optkeyr: OPTKEYR,
    /// Opaque SR register
    pub sr: SR,
    /// Opaque SR register
    pub c2sr: C2SR,
    /// Opaque SR register
    pub cr: CR,
    /// Opaque ECCR register
    pub eccr: ECCR,
    /// Opaque PCROP1SR register
    pub pcrop1asr: PCROP1ASR,
    /// Opaque PCROP1ER register
    pub pcrop1aer: PCROP1AER,
    /// Opaque WRP1AR register
    pub wrp1ar: WRP1AR,
    /// Opaque WRP1BR register
    pub wrp1br: WRP1BR,
}

macro_rules! generate_register {
    ($a:ident, $b:ident, $name:expr) => {
        #[doc = "Opaque "]
        #[doc = $name]
        #[doc = " register"]
        pub struct $a;

        impl $a {
            #[allow(unused)]
            pub(crate) fn $b(&mut self) -> &flash::$a {
                // NOTE(unsafe) this proxy grants exclusive access to this register
                unsafe { &(*FLASH::ptr()).$b }
            }
        }
    };

    ($a:ident, $b:ident) => {
        generate_register!($a, $b, stringify!($a));
    };
}

generate_register!(ACR, acr);
generate_register!(KEYR, keyr);
generate_register!(OPTKEYR, optkeyr);
generate_register!(SR, sr);
generate_register!(C2SR, c2sr);
generate_register!(CR, cr);
generate_register!(ECCR, eccr);
generate_register!(PCROP1ASR, pcrop1asr);
generate_register!(PCROP1AER, pcrop1aer);
generate_register!(WRP1AR, wrp1ar);
generate_register!(WRP1BR, wrp1br);

const FLASH_KEY1: u32 = 0x4567_0123;
const FLASH_KEY2: u32 = 0xCDEF_89AB;

impl KEYR {
    /// Unlock the flash registers via KEYR to access the flash programming
    pub fn unlock_flash<'a>(
        &'a mut self,
        sr: &'a mut SR,
        c2sr: &'a mut C2SR,
        cr: &'a mut CR,
    ) -> Result<FlashProgramming<'a>, Error> {
        let keyr = self.keyr();
        unsafe {
            keyr.write(|w| w.bits(FLASH_KEY1));
            keyr.write(|w| w.bits(FLASH_KEY2));
        }

        if cr.cr().read().lock().bit_is_clear() {
            Ok(FlashProgramming { sr, c2sr, cr })
        } else {
            Err(Error::Failure)
        }
    }
}

impl FlashPage {
    const SIZE: usize = 4096;

    /// This gives the starting address of a flash page in physical address
    pub const fn to_address(&self) -> usize {
        0x0800_0000 + self.0 * Self::SIZE
    }
}

/// Flash programming interface
pub struct FlashProgramming<'a> {
    sr: &'a mut SR,
    c2sr: &'a mut C2SR,
    cr: &'a mut CR,
}

impl<'a> Drop for FlashProgramming<'a> {
    fn drop(&mut self) {
        // Lock on drop
        self.lock();
    }
}

impl<'a> Read for FlashProgramming<'a> {
    type NativeType = u8;

    #[inline]
    fn read_native(&self, address: usize, array: &mut [Self::NativeType]) {
        let mut address = address as *const Self::NativeType;

        for data in array {
            unsafe {
                *data = ptr::read(address);
                address = address.add(1);
            }
        }
    }

    #[inline]
    fn read(&self, address: usize, buf: &mut [u8]) {
        self.read_native(address, buf);
    }
}

impl<'a> WriteErase for FlashProgramming<'a> {
    type NativeType = u64;

    fn status(&self) -> flash_trait::Result {
        let sr = unsafe { &(*FLASH::ptr()).sr }.read();

        if sr.bsy().bit_is_set() {
            Err(flash_trait::Error::Busy)
        } else if sr.pgaerr().bit_is_set() || sr.progerr().bit_is_set() || sr.wrperr().bit_is_set()
        {
            Err(flash_trait::Error::Illegal)
        } else {
            Ok(())
        }
    }

    fn erase_page(&mut self, page: flash_trait::FlashPage) -> flash_trait::Result {
        self.cr
            .cr()
            .modify(|_, w| unsafe { w.pnb().bits(page.0 as u8).per().set_bit() });

        self.cr.cr().modify(|_, w| w.strt().set_bit());

        let res = self.wait();

        self.cr.cr().modify(|_, w| w.per().clear_bit());

        res
    }

    fn write_native(&mut self, address: usize, array: &[Self::NativeType]) -> flash_trait::Result {
        // NB: The check for alignment of the address, and that the flash is erased is made by the
        // flash controller. The `wait` function will return the proper error codes.
        let mut address = address as *mut u32;

        self.cr.cr().modify(|_, w| w.pg().set_bit());

        for dword in array {
            unsafe {
                ptr::write_volatile(address, *dword as u32);
                ptr::write_volatile(address.add(1), (*dword >> 32) as u32);

                address = address.add(2);
            }

            self.wait()?;

            if self.sr.sr().read().eop().bit_is_set() {
                self.sr.sr().modify(|_, w| w.eop().clear_bit());
            }
        }

        self.cr.cr().modify(|_, w| w.pg().clear_bit());

        Ok(())
    }

    fn write(&mut self, address: usize, data: &[u8]) -> flash_trait::Result {
        let address_offset = address % mem::align_of::<Self::NativeType>();
        let unaligned_size = (mem::size_of::<Self::NativeType>() - address_offset)
            % mem::size_of::<Self::NativeType>();

        if unaligned_size > 0 {
            let unaligned_data = &data[..unaligned_size];
            // Handle unaligned address data, make it into a native write
            let mut data = 0xffff_ffff_ffff_ffffu64;
            for b in unaligned_data {
                data = (data >> 8) | ((*b as Self::NativeType) << 56);
            }

            let unaligned_address = address - address_offset;
            let native = &[data];
            self.write_native(unaligned_address, native)?;
        }

        // Handle aligned address data
        let aligned_data = &data[unaligned_size..];
        let mut aligned_address = if unaligned_size > 0 {
            address - address_offset + mem::size_of::<Self::NativeType>()
        } else {
            address
        };

        let mut chunks = aligned_data.chunks_exact(mem::size_of::<Self::NativeType>());

        while let Some(exact_chunk) = chunks.next() {
            // Write chunks
            let native = &[Self::NativeType::from_ne_bytes(
                exact_chunk.try_into().unwrap(),
            )];
            self.write_native(aligned_address, native)?;
            aligned_address += mem::size_of::<Self::NativeType>();
        }

        let rem = chunks.remainder();

        if !rem.is_empty() {
            let mut data = 0xffff_ffff_ffff_ffffu64;
            // Write remainder
            for b in rem.iter().rev() {
                data = (data << 8) | *b as Self::NativeType;
            }

            let native = &[data];
            self.write_native(aligned_address, native)?;
        }

        Ok(())
    }
}

impl<'a> FlashProgramming<'a> {
    /// Lock the flash memory controller
    fn lock(&mut self) {
        self.cr.cr().modify(|_, w| w.lock().set_bit());
    }

    /// Wait till last flash operation is complete
    fn wait(&mut self) -> flash_trait::Result {
        while self.sr.sr().read().bsy().bit_is_set() || self.c2sr.c2sr().read().bsy().bit_is_set() {
        }

        self.status()
    }

    /// Erase all flash pages, note that this will erase the current running program if it is not
    /// called from a program running in RAM.
    pub fn erase_all_pages(&mut self) -> flash_trait::Result {
        self.cr.cr().modify(|_, w| w.mer().set_bit());
        self.cr.cr().modify(|_, w| w.strt().set_bit());

        let res = self.wait();

        self.cr.cr().modify(|_, w| w.mer().clear_bit());

        res
    }
}
