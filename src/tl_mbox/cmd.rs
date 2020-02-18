use crate::tl_mbox::PacketHeader;
use core::fmt::{Error, Formatter};

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Cmd {
    cmd_code: u16,
    payload_len: u8,
    payload: [u8; 255],
}

impl core::fmt::Debug for Cmd {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let cmd_code = self.clone().cmd_code;

        write!(
            f,
            "Cmd ({}, {}, [{}...])",
            cmd_code,
            self.payload_len,
            self.payload[0]
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
