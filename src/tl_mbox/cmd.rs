use crate::tl_mbox::PacketHeader;
use core::fmt::{Error, Formatter};

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Cmd {
    pub cmd_code: u16,
    pub payload_len: u8,
    pub payload: [u8; 255],
}

impl core::fmt::Debug for Cmd {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let cmd_code = self.clone().cmd_code;

        write!(
            f,
            "Cmd ({}, {}, [{}...])",
            cmd_code, self.payload_len, self.payload[0]
        )
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct CmdSerial {
    pub ty: u8,
    pub cmd: Cmd,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct CmdPacket {
    pub header: PacketHeader,
    pub cmdserial: CmdSerial,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct AclDataSerial {
    pub ty: u8,
    pub handle: u16,
    pub length: u16,
    pub acl_data: [u8; 1],
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct AclDataPacket {
    pub header: PacketHeader,
    pub acl_data_serial: AclDataSerial,
}
