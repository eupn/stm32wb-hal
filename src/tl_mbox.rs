use core::mem::MaybeUninit;
use crate::ipcc::IpccExt;
use crate::rcc::Rcc;

/**
 * Version
 * [0:3]   = Build - 0: Untracked - 15:Released - x: Tracked version
 * [4:7]   = branch - 0: Mass Market - x: ...
 * [8:15]  = Subversion
 * [16:23] = Version minor
 * [24:31] = Version major
 *
 * Memory Size
 * [0:7]   = Flash ( Number of 4k sector)
 * [8:15]  = Reserved ( Shall be set to 0 - may be used as flash extension )
 * [16:23] = SRAM2b ( Number of 1k sector)
 * [24:31] = SRAM2a ( Number of 1k sector)
 */
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct SafeBootInfoTable {
    version: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct RssInfoTable {
    version: u32,
    memory_size: u32,
    rss_info: u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct WirelessFwInfoTable {
    version: u32,
    memory_size: u32,
    thread_info: u32,
    ble_info: u32,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct DeviceInfoTable {
    safe_boot_info_table: SafeBootInfoTable,
    rss_info_table: RssInfoTable,
    wireless_fw_info_table: WirelessFwInfoTable,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct BleTable {
    pcmd_buffer: *const u8,
    pcs_buffer: *const u8,
    pevt_queue: *const u8,
    phci_acl_data_buffer: *const u8,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct ThreadTable {
    nostack_buffer: *const u8,
    clicmdrsp_buffer: *const u8,
    otcmdrsp_buffer: *const u8,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct SysTable {
    pcmd_buffer: *const u8,
    sys_queue: *const u8,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct MemManagerTable {
    spare_ble_buffer: *const u8,
    spare_sys_buffer: *const u8,

    blepool: *const u8,
    blepoolsize: u32,

    pevt_free_buffer_queue: *const u8,

    traces_evt_pool: *const u8,
    tracespoolsize: u32,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct TracesTable {
    traces_queue: *const u8,
}

#[derive(Debug)]
#[repr(C, align(4))]
struct Mac802154Table {
    p_cmdrsp_buffer: *const u8,
    p_notack_buffer: *const u8,
    evt_queue: *const u8,
}

/// Reference table. Contains pointers to all other tables.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct RefTable {
    device_info_table: *const DeviceInfoTable,
    ble_table: *const BleTable,
    thread_table: *const ThreadTable,
    sys_table: *const SysTable,
    mem_manager_table: *const MemManagerTable,
    traces_table: *const TracesTable,
    mac_802_15_4_table: *const Mac802154Table,
}

#[link_section = "S_TL_REF_TABLE"]
static mut TL_REF_TABLE: MaybeUninit<RefTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
#[no_mangle]
static mut TL_DEVICE_INFO_TABLE: MaybeUninit<DeviceInfoTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_BLE_TABLE: MaybeUninit<BleTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_THREAD_TABLE: MaybeUninit<ThreadTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_SYS_TABLE: MaybeUninit<SysTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_MEM_MANAGER_TABLE: MaybeUninit<MemManagerTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_TRACES_TABLE: MaybeUninit<TracesTable> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TL_MAC_802_15_4_TABLE: MaybeUninit<Mac802154Table> = MaybeUninit::uninit();

#[derive(Debug)]
#[repr(C, align(4))]
struct LinkedListNode {
    next: *const LinkedListNode,
    prev: *const LinkedListNode,
}

#[link_section = "MB_MEM1"]
static mut FREE_BUF_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM1"]
static mut TRACES_EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[repr(C, packed)]
struct PacketHeader {
    next: *const u32,
    prev: *const u32,
}

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct CsEvent {
    status: u8,
    numcmd: u8,
    cmdcode: u16,
}

const TL_PACKET_HEADER_SIZE: usize = core::mem::size_of::<PacketHeader>();
const TL_EVT_HEADER_SIZE: usize = 3;
const TL_CS_EVT_SIZE: usize = core::mem::size_of::<CsEvent>();

#[link_section = "MB_MEM2"]
static mut CS_BUFFER: MaybeUninit<[u8; TL_PACKET_HEADER_SIZE + TL_EVT_HEADER_SIZE + TL_CS_EVT_SIZE]> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

#[link_section = "MB_MEM2"]
static mut SYSTEM_EVT_QUEUE: MaybeUninit<LinkedListNode> = MaybeUninit::uninit();

pub mod cpu1_channels {
    use crate::ipcc::IpccChannel;

    pub const IPCC_BLE_CMD_CHANNEL: IpccChannel = IpccChannel::Channel1;
    pub const IPCC_SYSTEM_CMD_RSP_CHANNEL: IpccChannel = IpccChannel::Channel2;
    pub const IPCC_THREAD_OT_CMD_RSP_CHANNEL: IpccChannel = IpccChannel::Channel3;
    pub const IPCC_MAC_802_15_4_CMD_RSP_CHANNEL: IpccChannel = IpccChannel::Channel3;
    pub const IPCC_THREAD_CLI_CMD_CHANNEL: IpccChannel = IpccChannel::Channel5;
    pub const IPCC_MM_RELEASE_BUFFER_CHANNEL: IpccChannel = IpccChannel::Channel4;
    pub const IPCC_HCI_ACL_DATA_CHANNEL: IpccChannel = IpccChannel::Channel6;
}

pub mod cpu2_channels {
    use crate::ipcc::IpccChannel;

    pub const HW_IPCC_BLE_EVENT_CHANNEL: IpccChannel = IpccChannel::Channel1;
    pub const HW_IPCC_SYSTEM_EVENT_CHANNEL: IpccChannel = IpccChannel::Channel2;
    pub const HW_IPCC_THREAD_NOTIFICATION_ACK_CHANNEL: IpccChannel = IpccChannel::Channel3;
    pub const HW_IPCC_MAC_802_15_4_NOTIFICATION_ACK_CHANNEL: IpccChannel = IpccChannel::Channel3;
    pub const HW_IPCC_TRACES_CHANNEL: IpccChannel = IpccChannel::Channel4;
    pub const HW_IPCC_THREAD_CLI_NOTIFICATION_ACK_CHANNEL: IpccChannel = IpccChannel::Channel5;
}

/// Initializes low-level transport between CPU1 and BLE stack on CPU2.
pub fn tl_init(rcc: &mut crate::rcc::Rcc, ipcc: &mut crate::ipcc::Ipcc) {
    // Populate reference table with pointers in the shared memory
    unsafe {
        TL_REF_TABLE = MaybeUninit::new(RefTable {
            device_info_table: TL_DEVICE_INFO_TABLE.as_ptr(),
            ble_table: TL_BLE_TABLE.as_ptr(),
            thread_table: TL_THREAD_TABLE.as_ptr(),
            sys_table: TL_SYS_TABLE.as_ptr(),
            mem_manager_table: TL_MEM_MANAGER_TABLE.as_ptr(),
            traces_table: TL_TRACES_TABLE.as_ptr(),
            mac_802_15_4_table: TL_MAC_802_15_4_TABLE.as_ptr(),
        });
    }

    ipcc.init(rcc);
}