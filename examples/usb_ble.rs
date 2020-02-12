//! CDC-ACM serial port example using cortex-m-rtfm.
#![no_main]
#![no_std]
#![allow(non_snake_case)]

extern crate panic_semihosting;
extern crate stm32wb_hal as hal;

use rtfm::app;

use hal::flash::FlashExt;
use hal::prelude::*;
use hal::rcc::{ApbDivider, Config, HDivider, HseDivider, PllConfig, PllSrc, SysClkSrc, UsbClkSrc};
use hal::usb::{Peripheral, UsbBus, UsbBusType};
use hal::tl_mbox;

use usb_device::prelude::*;
use usb_device::bus;
use usb_device::device::UsbDevice;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[app(device = stm32wb_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, UsbBusType>,
        serial: SerialPort<'static, UsbBusType>,
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

        tl_mbox::tl_init(&mut rcc, &mut dp.IPCC.constrain());

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

        init::LateResources { usb_dev, serial }
    }

    #[task(binds = USB_HP, resources = [usb_dev, serial])]
    fn usb_tx(mut cx: usb_tx::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.serial);
    }

    #[task(binds = USB_LP, resources = [usb_dev, serial])]
    fn usb_rx0(mut cx: usb_rx0::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.serial);
    }

    #[task(binds = IPCC_C1_RX_IT, resources = [])]
    fn mbox_rx(cx: mbox_rx::Context) {
        cortex_m::asm::bkpt();
    }

    #[task(binds = IPCC_C1_TX_IT, resources = [])]
    fn mbox_tx(cx: mbox_tx::Context) {
        cortex_m::asm::bkpt();
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
