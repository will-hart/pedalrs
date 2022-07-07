#![no_std]
#![no_main]

#[cfg(feature = "stm32f0")]
use stm32f0xx_hal as hal;

#[cfg(feature = "stm32f1")]
use stm32f1xx_hal as hal;

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use panic_halt as _;

use hal::{pac::interrupt, prelude::*, usb::UsbBus};

use crate::{stateful_key::StatefulKey, usb_interface::UsbInterface};
use configure::{configure_gpio, Configuration, GpioConfiguration, TimerType};

mod configure;
mod stateful_key;
mod usb_descriptor;
mod usb_interface;

static FORCE_UPDATE: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
static TIMER: Mutex<RefCell<Option<TimerType>>> = Mutex::new(RefCell::new(None));

const UPDATE_LEFT_KEY_COMMAND: u8 = 0x19u8;
const UPDATE_RIGHT_KEY_COMMAND: u8 = 0x1Au8;
const UPDATE_SHIFT_KEY_SETTING_COMMAND: u8 = 0x1Bu8;
const RESET_CONFIG_COMMNAD: u8 = 0x1Cu8;

#[entry]
fn main() -> ! {
    let GpioConfiguration {
        btn_left,
        btn_right,
        peripheral,
        mut delay,
        timer,

        #[cfg(feature = "stm32f1")]
        mut flash,
    } = match configure_gpio() {
        Some(config) => config,
        None => panic!("Error configuring GPIO"),
    };

    cortex_m::interrupt::free(|cs| {
        TIMER.borrow(cs).replace(Some(timer));
    });

    /* Load configuration */
    #[cfg(feature = "stm32f1")]
    let mut config = Configuration::read(&mut flash);
    #[cfg(feature = "stm32f0")]
    let mut config = Configuration::default();

    /* create the StatefulKeys */
    let mut key_left = StatefulKey::new(btn_left, config.left_key);
    let mut key_right = StatefulKey::new(btn_right, config.right_key);

    let alloc = UsbBus::new(peripheral);
    let mut usb = UsbInterface::new(&alloc);

    // Main loop
    loop {
        if usb.poll() {
            match usb.read_command() {
                Ok((data, num_read)) => {
                    if num_read > 0 {
                        match data[0] {
                            UPDATE_LEFT_KEY_COMMAND => {
                                config.left_key = data[1];
                            }
                            UPDATE_RIGHT_KEY_COMMAND => {
                                config.right_key = data[1];
                            }
                            UPDATE_SHIFT_KEY_SETTING_COMMAND => {
                                config.both_for_shift = data[1] > 0;
                            }
                            RESET_CONFIG_COMMNAD => {
                                config.left_key = 0x14u8;
                                config.right_key = 0x08u8;
                                config.both_for_shift = true;
                            }
                            _ => {}
                        }

                        #[cfg(feature = "stm32f1")]
                        config.write(&mut flash);

                        key_left.replace_keycode(config.left_key);
                        key_right.replace_keycode(config.right_key);
                    }
                }
                Err(e) => panic!("Error receiving USB data {:?}", e),
            };
        }

        // update the keys
        key_left.update();
        key_right.update();

        // update the USB, forcing an update if the timer has ticked
        let force_update = cortex_m::interrupt::free(|cs| FORCE_UPDATE.borrow(cs).replace(false));
        match usb.send_report(
            key_left.current_keycode(),
            key_right.current_keycode(),
            config.both_for_shift,
            force_update,
        ) {
            Ok(sent_data) => {
                if sent_data {
                    delay.delay_ms(5u8);
                }
            }
            Err(e) => panic!("Error sending via USB {:?}", e),
        }
    }
}

#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        *FORCE_UPDATE.borrow(cs).borrow_mut() = true;
        match TIMER.borrow(cs).borrow_mut().as_mut() {
            Some(timer) => {
                #[cfg(feature = "stm32f0")]
                timer.wait().ok();

                #[cfg(feature = "stm32f1")]
                timer.clear_interrupt(hal::timer::Event::Update);
            }
            None => todo!(),
        }
    });
}
