
pub enum TlPacketType {
    BleCmd = 0x01,
    AclData = 0x02,
    BleEvt = 0x04,

    OtCmd = 0x08,
    OtRsp = 0x09,
    CliCmd = 0x0A,
    OtNot = 0x0C,
    OtAck = 0x0D,
    CliNot = 0x0E,
    CliAck = 0x0F,

    SysCmd = 0x10,
    SysRsp = 0x11,
    SysEvt = 0x12,

    LocCmd = 0x20,
    LocRsp = 0x21,

    TracesApp = 0x40,
    TracesWl = 0x41,
}