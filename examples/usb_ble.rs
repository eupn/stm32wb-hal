//! HCI BLE transparent mode example to be used with CubeRFMon software.
#![no_main]
#![no_std]
#![allow(non_snake_case)]

extern crate panic_semihosting;
extern crate stm32wb_hal as hal;

use cortex_m_rt::exception;

use rtfm::app;

use hal::flash::FlashExt;
use hal::prelude::*;
use hal::rcc::{
    ApbDivider, Config, HDivider, HseDivider, PllConfig, PllSrc, SysClkSrc, UsbClkSrc,
};
use hal::usb::{Peripheral, UsbBus, UsbBusType};

use hal::ipcc::Ipcc;
use hal::tl_mbox::evt::EvtBox;
use hal::tl_mbox::{TlMbox, WirelessFwInfoTable};
use usb_device::bus;
use usb_device::device::UsbDevice;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};
use hal::tl_mbox::shci::ShciBleInitCmdParam;

#[app(device = stm32wb_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, UsbBusType>,
        serial: SerialPort<'static, UsbBusType>,

        mbox: TlMbox,
        ipcc: Ipcc,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        let dp = cx.device;
        let rcc = dp.RCC.constrain();

        // Fastest clock configuration.
        // * 32 MHz HSE with PLL
        // * 64 MHz CPU1, 32 MHz CPU2
        // * 64 MHz for APB1, APB2
        // * USB clock source from PLLQ (32 / 2 * 3 = 48)
        let clock_config = Config::new(SysClkSrc::Pll(PllSrc::Hse(HseDivider::NotDivided)))
            .cpu1_hdiv(HDivider::NotDivided)
            .cpu2_hdiv(HDivider::Div2)
            .apb1_div(ApbDivider::NotDivided)
            .apb2_div(ApbDivider::NotDivided)
            .pll_cfg(PllConfig {
                m: 2,
                n: 12,
                r: 3,
                q: Some(4),
                p: Some(3),
            })
            .usb_src(UsbClkSrc::PllQ);

        let mut rcc = rcc.apply_clock_config(clock_config, &mut dp.FLASH.constrain().acr);

        let mut ipcc = dp.IPCC.constrain();
        let mbox = TlMbox::tl_init(&mut rcc, &mut ipcc);

        // Boot CPU2
        hal::pwr::set_cpu2(true);

        // Enable USB power supply
        hal::pwr::set_usb(true);

        let mut gpioa = dp.GPIOA.split(&mut rcc);

        let usb = Peripheral {
            usb: dp.USB,
            pin_dm: gpioa.pa11.into_af10(&mut gpioa.moder, &mut gpioa.afrh),
            pin_dp: gpioa.pa12.into_af10(&mut gpioa.moder, &mut gpioa.afrh),
        };

        *USB_BUS = Some(UsbBus::new(usb));

        let serial = SerialPort::new(USB_BUS.as_ref().unwrap());

        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Fake company")
            .product("Serial port")
            .serial_number("TEST")
            .device_class(USB_CLASS_CDC)
            .build();

        init::LateResources {
            usb_dev,
            serial,
            mbox,
            ipcc,
        }
    }

    #[task(binds = USB_HP, resources = [usb_dev, serial])]
    fn usb_tx(mut cx: usb_tx::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.serial);
    }

    #[task(binds = USB_LP, resources = [usb_dev, serial])]
    fn usb_rx0(mut cx: usb_rx0::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.serial);
    }

    #[task(binds = IPCC_C1_RX_IT, resources = [mbox, ipcc], spawn = [evt])]
    fn mbox_rx(mut cx: mbox_rx::Context) {
        cx.resources
            .mbox
            .interrupt_ipcc_rx_handler(&mut cx.resources.ipcc);

        while let Some(evt) = cx.resources.mbox.dequeue_event() {
            cx.spawn.evt(evt).unwrap();
        }
    }

    #[task(binds = IPCC_C1_TX_IT, resources = [mbox, ipcc])]
    fn mbox_tx(mut cx: mbox_tx::Context) {
        cx.resources
            .mbox
            .interrupt_ipcc_tx_handler(&mut cx.resources.ipcc);
    }

    #[task(resources = [mbox, ipcc])]
    fn evt(mut cx: evt::Context, evt: EvtBox) {
        let ipcc = &mut cx.resources.ipcc;
        let event = evt.evt();
        cortex_m_semihosting::hprintln!("Got event #{}", event.kind()).unwrap();

        if event.kind() == 18 {
            // This is so slow with semihosting that it's blocking the USB device discovery
            /*if let Some(fw_info) = cx.resources.mbox.wireless_fw_info() {
                let fw_info: WirelessFwInfoTable = fw_info;

                cortex_m_semihosting::hprintln!("-- CPU2 wireless firmware info --").unwrap();
                cortex_m_semihosting::hprintln!(
                    "FW version: {}.{}.{}",
                    fw_info.version_major(),
                    fw_info.version_minor(),
                    fw_info.subversion()
                )
                .unwrap();
                cortex_m_semihosting::hprintln!(
                    "FLASH size: {} KB",
                    fw_info.flash_size() as u32 * 4096 / 1024
                )
                .unwrap();
                cortex_m_semihosting::hprintln!(
                    "SRAM2a size {} KB",
                    fw_info.sram2a_size() as u32 * 1024
                )
                .unwrap();
                cortex_m_semihosting::hprintln!(
                    "SRAM2b size {} KB",
                    fw_info.sram2b_size() as u32 * 1024
                )
                .unwrap();
            }*/
            
            let param = ShciBleInitCmdParam {
                p_ble_buffer_address: core::ptr::null(),
                ble_buffer_size: 0,
                num_attr_record: 68,
                num_attr_serv: 8,
                attr_value_arr_size: 1344,
                num_of_links: 8,
                extended_packet_length_enable: 1,
                pr_write_list_size: 0x3A,
                mb_lock_count: 0x79,
                att_mtu: 156,
                slave_sca: 500,
                master_sca: 0,
                ls_source: 1,
                max_conn_event_length: 0xFFFFFFFF,
                hs_startup_time: 0x148,
                viterbi_enable: 1,
                ll_only: 0,
                hw_version: 0
            };

            hal::tl_mbox::shci::shci_ble_init(ipcc, param);
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART1();
    }
};

fn usb_poll<B: bus::UsbBus>(
    usb_dev: &mut UsbDevice<'static, B>,
    serial: &mut SerialPort<'static, B>,
) {
    if !usb_dev.poll(&mut [serial]) {
        return;
    }

    let mut buf = [0u8; 8];

    match serial.read(&mut buf) {
        Ok(count) if count > 0 => {
            // Echo back in upper case
            for c in buf[0..count].iter_mut() {
                if 0x61 <= *c && *c <= 0x7a {
                    *c &= !0x20;
                }
            }

            serial.write(&buf[0..count]).ok();
        }
        _ => {}
    }
}

#[exception]
fn DefaultHandler(irqn: i16) -> ! {
    panic!("Unhandled IRQ: {}", irqn);
}
