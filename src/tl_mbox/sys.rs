//! IPCC SYS (System) channel routines.

use super::{channels, unsafe_linked_list};
use crate::ipcc::{Ipcc, IpccChannel};
use crate::tl_mbox::cmd::CmdPacket;
use crate::tl_mbox::{SysTable, EVT_QUEUE, SYSTEM_EVT_QUEUE};
use core::borrow::{Borrow, BorrowMut};
use core::pin::Pin;

pub type SysCallback = fn();

pub struct Sys {
    config: Config,
}

pub struct Config {
    pub cmd_evt_cb: SysCallback,
    pub sys_evt_cb: SysCallback,
}

unsafe impl Send for Config {}
unsafe impl Sync for Config {}

impl Sys {
    pub fn new(ipcc: &mut Ipcc, config: Config, system_cmd_buffer: *const CmdPacket) -> Self {
        cortex_m_semihosting::hprintln!("CMD buffer: {:?}", system_cmd_buffer);

        ipcc.c1_set_rx_channel(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL, true);

        unsafe {
            unsafe_linked_list::LST_init_head(SYSTEM_EVT_QUEUE.as_mut_ptr());

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
        (self.config.cmd_evt_cb)();
    }

    pub fn evt_handler(&self, ipcc: &mut Ipcc) {
        (self.config.sys_evt_cb)();
        ipcc.c1_clear_flag_channel(channels::cpu2::IPCC_SYSTEM_EVENT_CHANNEL);
    }
}
