//! IPCC SYS (System) channel routines.

use super::channels;
use crate::ipcc::Ipcc;
use crate::tl_mbox::cmd::CmdPacket;
use crate::tl_mbox::{SysTable, SYSTEM_EVT_QUEUE, evt, EventCallback};
use crate::tl_mbox::unsafe_linked_list::{LinkedListNode, LST_is_empty, LST_init_head, LST_remove_head};
use crate::tl_mbox::evt::EvtBox;

pub type SysCallback = fn();

pub struct Sys {
    config: Config,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub cmd_evt_cb: SysCallback,
    pub sys_evt_cb: SysCallback,
}

unsafe impl Send for Config {}
unsafe impl Sync for Config {}

impl Sys {
    pub fn new(ipcc: &mut Ipcc, config: Config, system_cmd_buffer: *const CmdPacket) -> Self {
        ipcc.c1_set_rx_channel(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL, true);

        unsafe {
            LST_init_head(SYSTEM_EVT_QUEUE.as_mut_ptr());

            *super::TL_SYS_TABLE.as_mut_ptr() = SysTable {
                pcmd_buffer: system_cmd_buffer,
                sys_queue: SYSTEM_EVT_QUEUE.as_ptr(),
            };
        }

        Sys { config }
    }

    pub fn send_cmd(&self, ipcc: &mut Ipcc) {
        ipcc.c1_set_flag_channel(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL);
        ipcc.c1_set_tx_channel(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, true);
    }

    pub fn cmd_evt_handler(&self, ipcc: &mut Ipcc) {
        ipcc.c1_set_tx_channel(channels::cpu1::IPCC_SYSTEM_CMD_RSP_CHANNEL, false);

        // TODO: handle system cmd event
    }

    pub fn evt_handler(&self, ipcc: &mut Ipcc, cb: EventCallback) {
        unsafe {
            let mut node_ptr: *mut LinkedListNode = core::ptr::null_mut();
            let node_ptr_ptr: *mut *mut LinkedListNode = &mut node_ptr;

            while !LST_is_empty(SYSTEM_EVT_QUEUE.as_mut_ptr()) {
                LST_remove_head(SYSTEM_EVT_QUEUE.as_mut_ptr(), node_ptr_ptr);

                let event = core::mem::transmute::<*mut LinkedListNode, *const evt::EvtPacket>(node_ptr);
                (cb)(EvtBox::new(event))
            }
        }

        ipcc.c1_clear_flag_channel(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL);
    }
}
