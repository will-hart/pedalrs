#![no_std]
#![no_main]

use configure::GpioConfiguration;
use panic_halt as _;
use stm32f1xx_hal as hal;

use cortex_m_rt::entry;
use hal::{prelude::*, usb::UsbBus};

mod configure;
mod stateful_key;
mod usb_descriptor;
mod usb_interface;
use crate::{stateful_key::StatefulKey, usb_interface::UsbInterface};

#[entry]
fn main() -> ! {
    let GpioConfiguration {
        btn_left,
        btn_right,
        peripheral,
        mut delay,
    } = match configure::configure_gpio() {
        Some(config) => config,
        None => panic!("Error configuring GPIO"),
    };

    /* create the StatefulKeys */
    let mut key_left = StatefulKey::new(btn_left, 0x14_u8);
    let mut key_right = StatefulKey::new(btn_right, 0x08_u8);

    let alloc = UsbBus::new(peripheral);
    let mut usb = UsbInterface::new(&alloc);

    // Main loop
    loop {
        if usb.poll() {
            match usb.read_command() {
                Ok((data, num_read)) => {
                    if num_read > 0 {
                        match if data[0] == 0x19 {
                            Some(&mut key_left)
                        } else if data[0] == 0x1A {
                            Some(&mut key_right)
                        } else {
                            None
                        } {
                            Some(key) => {
                                key.replace_keycode(data[1]);
                            }
                            None => {}
                        }
                    }
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
                    delay.delay_ms(5u8);
                }
            }
            Err(e) => panic!("Error sending via USB {:?}", e),
        }
    }
}
