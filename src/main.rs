#![no_std]
#![no_main]

use configure::{configure_gpio, GpioConfiguration};
use eeprom24x::{Eeprom24x, SlaveAddr};
use panic_halt as _;

#[cfg(feature = "stm32f0")]
use stm32f0xx_hal as hal;

#[cfg(feature = "stm32f1")]
use stm32f1xx_hal as hal;

use cortex_m_rt::entry;
use hal::{prelude::*, usb::UsbBus};

mod configure;
mod stateful_key;
mod usb_descriptor;
mod usb_interface;
use crate::{stateful_key::StatefulKey, usb_interface::UsbInterface};

static LEFT_KEY_ADDRESS: u32 = 0x111;
static RIGHT_KEY_ADDRESS: u32 = 0x121;

#[entry]
fn main() -> ! {
    let GpioConfiguration {
        btn_left,
        btn_right,
        peripheral,
        mut delay,
        sda,
        scl,
        timer,
    } = match configure_gpio() {
        Some(config) => config,
        None => panic!("Error configuring GPIO"),
    };

    /* Create EEPROM for config */
    let i2c = bitbang_hal::i2c::I2cBB::new(scl, sda, timer);
    let mut eeprom = Eeprom24x::new_24x04(i2c, SlaveAddr::default());

    /* Read the initial config */
    let left_keycode = eeprom.read_byte(LEFT_KEY_ADDRESS).unwrap_or(0x14u8);
    let right_keycode = eeprom.read_byte(RIGHT_KEY_ADDRESS).unwrap_or(0x08u8);

    /* create the StatefulKeys */
    let mut key_left = StatefulKey::new(btn_left, left_keycode);
    let mut key_right = StatefulKey::new(btn_right, right_keycode);

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

                                // ensure the EEPROM has the latest keys
                                eeprom
                                    .write_byte(LEFT_KEY_ADDRESS, key_left.get_code())
                                    .unwrap();
                                eeprom
                                    .write_byte(RIGHT_KEY_ADDRESS, key_right.get_code())
                                    .unwrap();
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
