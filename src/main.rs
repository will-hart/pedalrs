#![no_std]
#![no_main]

use panic_halt as _;
use stm32f0xx_hal as hal;

use cortex_m::asm::delay as cycle_delay;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use hal::{
    delay::Delay,
    pac::{self},
    prelude::*,
    rcc::HSEBypassMode,
    usb::{Peripheral, UsbBus},
};
use switch_hal::IntoSwitch;

mod stateful_key;
mod usb_interface;
use crate::stateful_key::StatefulKey;
use crate::usb_interface::UsbInterface;

#[entry]
fn main() -> ! {
    /* Get access to device and core peripherals */
    let mut dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    /* remap USB pins */
    stm32f0xx_hal::usb::remap_pins(&mut dp.RCC, &mut dp.SYSCFG);

    /* Set up sysclk and freeze it */
    let mut clocks = dp
        .RCC
        .configure()
        .hse(16.mhz(), HSEBypassMode::Bypassed)
        .sysclk(48.mhz())
        .pclk(24.mhz())
        .freeze(&mut dp.FLASH);

    /* Set up systick delay */
    let mut delay = Delay::new(cp.SYST, &clocks);

    /* Set up GPIO */
    let gpioa = dp.GPIOA.split(&mut clocks);
    let gpiob = dp.GPIOB.split(&mut clocks);

    /* Set up "button" pins */
    // TODO: Get correct pins
    let (btn_left, btn_right, mut usb_dp, usb_dm) = cortex_m::interrupt::free(move |cs| {
        (
            gpioa
                .pa5
                .into_pull_up_input(cs)
                .downgrade()
                .into_active_low_switch(),
            gpiob
                .pb1
                .into_pull_up_input(cs)
                .downgrade()
                .into_active_low_switch(),
            gpioa.pa12.into_push_pull_output(cs),
            gpioa.pa11.into_floating_input(cs),
        )
    });

    /* create the StatefulKeys */
    let mut key_left = StatefulKey::new(btn_left, 0x14_u8);
    let mut key_right = StatefulKey::new(btn_right, 0x08_u8);

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    usb_dp.set_low().ok();
    cycle_delay(100); // >1 us, I think

    // now fire up the USB BUS
    let peripheral = Peripheral {
        usb: dp.USB,
        pin_dm: usb_dm,
        pin_dp: cortex_m::interrupt::free(move |cs| usb_dp.into_floating_input(cs)),
    };
    let alloc = UsbBus::new(peripheral);
    let mut usb = UsbInterface::new(&alloc);

    // Main loop
    loop {
        if usb.poll() {
            match usb.read() {
                Ok(data) => {
                    usb.set_response_bits(data[0], data[1]);
                    usb.send_report().ok();
                    delay.delay_ms(5u8);
                    usb.set_response_bits(0, 0);
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
