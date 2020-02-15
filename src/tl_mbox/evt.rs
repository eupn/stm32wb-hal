use crate::tl_mbox::PacketHeader;

/**
 * The payload of `Evt` for a command status event
 */
#[repr(C, packed)]
pub struct CsEvt {
    status: u8,
    num_cmd: u8,
    cmd_code: u16,
}

/**
 * The payload of `Evt` for a command complete event
 */
#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct CcEvt {
    num_cmd: u8,
    cmd_code: u16,
    payload: [u8; 1],
}

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct AsynchEvt {
    sub_evt_code: u16,
    payload: [u8; 1],
}

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct Evt {
    evt_code: u8,
    payload_len: u8,
    payload: [u8; 1],
}

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct EvtSerial {
    kind: u8,
    evt: Evt,
}

/**
 * This format shall be used for all events (asynchronous and command response) reported
 * by the CPU2 except for the command response of a system command where the header is not there
 * and the format to be used shall be `EvtSerial`.
 * Note: Be careful that the asynchronous events reported by the CPU2 on the system channel do
 * include the header and shall use `EvtPacket` format. Only the command response format on the
 * system channel is different.
 */
#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct EvtPacket {
    header: PacketHeader,
    evt_serial: EvtSerial,
}

impl EvtPacket {
    pub fn kind(&self) -> u8 {
        self.evt_serial.kind
    }

    pub fn evt(&self) -> &Evt {
        &self.evt_serial.evt
    }
}