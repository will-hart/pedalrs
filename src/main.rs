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

mod configure;
mod stateful_key;
mod usb_descriptor;
mod usb_interface;
use crate::{stateful_key::StatefulKey, usb_interface::UsbInterface};
use configure::{configure_gpio, GpioConfiguration, TimerType};

static FORCE_UPDATE: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));
static TIMER: Mutex<RefCell<Option<TimerType>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let GpioConfiguration {
        btn_left,
        btn_right,
        peripheral,
        mut delay,
        timer,
    } = match configure_gpio() {
        Some(config) => config,
        None => panic!("Error configuring GPIO"),
    };

    cortex_m::interrupt::free(|cs| {
        TIMER.borrow(cs).replace(Some(timer));
    });

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

        // update the keys
        key_left.update();
        key_right.update();

        // update the USB, forcing an update if the timer has ticked
        let force_update = cortex_m::interrupt::free(|cs| FORCE_UPDATE.borrow(cs).replace(false));
        match usb.send_report(
            key_left.current_keycode(),
            key_right.current_keycode(),
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
