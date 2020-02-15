use crate::tl_mbox::PacketHeader;
use core::fmt::{Error, Formatter};

#[repr(C, packed)]
pub struct Cmd {
    cmdcode: u16,
    plen: u8,
    payload: [u8; 255],
}

impl core::fmt::Debug for Cmd {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "Cmd ({}, {}, [{}...])",
            self.cmdcode.clone(), self.plen, self.payload[0]
        )
    }
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct CmdSerial {
    ty: u8,
    cmd: Cmd,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct CmdPacket {
    header: PacketHeader,
    cmdserial: CmdSerial,
}
