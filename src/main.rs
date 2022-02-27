#![no_std]
#![no_main]

use panic_halt as _;
use stm32f0xx_hal as hal;

use cortex_m_rt::entry;
use hal::{prelude::*, usb::UsbBus};

mod configure;
mod stateful_key;
mod usb_descriptor;
mod usb_interface;
use crate::stateful_key::StatefulKey;
use crate::usb_interface::UsbInterface;

#[entry]
fn main() -> ! {
    let mut config = match configure::configure_gpio() {
        Some(config) => config,
        None => panic!("Error configuring GPIO"),
    };

    /* create the StatefulKeys */
    let mut key_left = StatefulKey::new(config.btn_left, 0x14_u8);
    let mut key_right = StatefulKey::new(config.btn_right, 0x08_u8);

    let alloc = UsbBus::new(config.peripheral);
    let mut usb = UsbInterface::new(&alloc);

    // Main loop
    loop {
        if usb.poll() {
            match usb.read() {
                Ok(_data) => {
                    // nop for now
                }
                Err(e) => panic!("Error receiving USB data {:?}", e),
            };
        }

        // update the USB
        usb.update_key(&mut key_left, 0);
        usb.update_key(&mut key_right, 1);

        match usb.send_report() {
            Ok(sent_data) => {
                if sent_data {
                    config.delay.delay_ms(5u8);
                }
            }
            Err(e) => panic!("Error sending via USB {:?}", e),
        }
    }
}
