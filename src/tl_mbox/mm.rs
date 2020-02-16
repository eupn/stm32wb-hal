//! MemoryManager routines.

use core::mem::MaybeUninit;

use super::unsafe_linked_list::{LinkedListNode, LST_init_head, LST_insert_tail, LST_remove_head, LST_is_empty};
use super::{FREE_BUF_QUEUE, LOCAL_FREE_BUF_QUEUE, TL_MEM_MANAGER_TABLE, MemManagerTable, BLE_SPARE_EVT_BUF, SYS_SPARE_EVT_BUF, EVT_POOL, POOL_SIZE};
use super::channels::cpu1::IPCC_MM_RELEASE_BUFFER_CHANNEL;

use crate::tl_mbox::evt::{EvtPacket, Evt};
use crate::ipcc::Ipcc;
use crate::tl_mbox::TL_REF_TABLE;

pub(super) struct MemoryManager {}

impl MemoryManager {
    pub fn new() -> Self {
        // Configure MemManager
        unsafe {
            LST_init_head(FREE_BUF_QUEUE.as_mut_ptr());
            LST_init_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr());

            TL_MEM_MANAGER_TABLE = MaybeUninit::new(MemManagerTable {
                spare_ble_buffer: BLE_SPARE_EVT_BUF.as_ptr().cast(),
                spare_sys_buffer: SYS_SPARE_EVT_BUF.as_ptr().cast(),
                blepool: EVT_POOL.as_ptr().cast(),
                blepoolsize: POOL_SIZE as u32,
                pevt_free_buffer_queue: FREE_BUF_QUEUE.as_mut_ptr(),
                traces_evt_pool: core::ptr::null(),
                tracespoolsize: 0,
            });
        }

        MemoryManager {}
    }
}

pub fn evt_drop(evt: *const EvtPacket, ipcc: &mut Ipcc) {
    cortex_m_semihosting::hprintln!("[mm] dropping event: {:?}", evt).unwrap();

    unsafe {
        let list_node = core::mem::transmute::<*const EvtPacket, _>(evt);

        LST_insert_tail(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), list_node);

        let channel_is_busy = ipcc.c1_is_active_flag(IPCC_MM_RELEASE_BUFFER_CHANNEL);

        // Postpone event buffer freeing to IPCC interrupt handler
        if channel_is_busy {
            ipcc.c1_set_tx_channel(IPCC_MM_RELEASE_BUFFER_CHANNEL, true);
        } else {
            send_free_buf();
            ipcc.c1_set_flag_channel(IPCC_MM_RELEASE_BUFFER_CHANNEL);
        }
    }
}

/// Gives free event buffers back to the CPU2 from local buffer queue.
pub fn send_free_buf() {
    cortex_m_semihosting::hprintln!("[mm] sending free buffer").unwrap();

    unsafe {
        let mut node_ptr: *mut LinkedListNode = core::ptr::null_mut();
        let node_ptr_ptr: *mut *mut LinkedListNode = &mut node_ptr;

        while !LST_is_empty(LOCAL_FREE_BUF_QUEUE.as_mut_ptr()) {
            LST_remove_head(LOCAL_FREE_BUF_QUEUE.as_mut_ptr(), node_ptr_ptr);
            LST_insert_tail((&*(*TL_REF_TABLE.as_ptr()).mem_manager_table).pevt_free_buffer_queue, node_ptr);
        }
    }
}

/// Free buffer channel interrupt handler.
pub fn free_buf_handler(ipcc: &mut Ipcc) {
    ipcc.c1_set_tx_channel(IPCC_MM_RELEASE_BUFFER_CHANNEL, false);
    send_free_buf();
    ipcc.c1_set_flag_channel(IPCC_MM_RELEASE_BUFFER_CHANNEL);
}