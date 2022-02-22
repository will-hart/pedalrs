#![no_std]
#![no_main]

use panic_halt as _;
use stm32f1xx_hal as hal;

use cortex_m::{asm::delay as cycle_delay, interrupt::free};
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use hal::{
    delay::Delay,
    pac::{self},
    prelude::*,
    usb::{Peripheral, UsbBus},
};
use switch_hal::{IntoSwitch, OutputSwitch};
use usb_device::{class_prelude::UsbBusAllocator, prelude::*};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::hid_class::HIDClass;

mod stateful_key;
use crate::stateful_key::StatefulKey;

static mut USB_ALLOC: Option<UsbBusAllocator<UsbBus<Peripheral>>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus<Peripheral>>> = None;
static mut USB_HID: Option<HIDClass<UsbBus<Peripheral>>> = None;
static mut KEY_BUFFER: [u8; 6] = [0, 0, 0, 0, 0, 0];

#[entry]
fn main() -> ! {
    /* Get access to device and core peripherals */
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    /* Get access to RCC, AFIO and GPIOA */
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    /* Set up sysclk and freeze it */
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);
    assert!(clocks.usbclk_valid());

    /* Set up systick delay */
    let mut delay = Delay::new(cp.SYST, clocks);

    /* Set up GPIO */
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    /* Set up "button" pins */
    let btn_left = gpioa
        .pa9
        .into_pull_up_input(&mut gpioa.crh)
        .downgrade()
        .into_active_low_switch();

    let btn_right = gpioa
        .pa10
        .into_pull_up_input(&mut gpioa.crh)
        .downgrade()
        .into_active_low_switch();

    /* create the StatefulKeys */
    let mut key_left = StatefulKey::new(btn_left, 0x14_u8);
    let mut key_right = StatefulKey::new(btn_right, 0x08_u8);

    /* Set up LED pin for status */
    let mut led = gpioc
        .pc13
        .into_push_pull_output(&mut gpioc.crh)
        .into_active_low_switch();
    led.off().ok();

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low().ok();
    cycle_delay(100); // >1 us, I think

    // now fire up the USB BUS
    let usb = Peripheral {
        usb: dp.USB,
        pin_dm: gpioa.pa11,
        pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
    };

    let usb_alloc = unsafe {
        USB_ALLOC = Some(UsbBus::new(usb));
        USB_ALLOC.as_ref().unwrap()
    };

    // create a device
    unsafe {
        USB_HID = Some(HIDClass::new(&usb_alloc, KeyboardReport::desc(), 63));
        USB_BUS = Some(
            UsbDeviceBuilder::new(&usb_alloc, UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("Atto Zepto")
                .product("Pedalrs")
                .serial_number("000001")
                .device_class(0x03) // HID device from usb.org device classes
                .build(),
        );
    };

    loop {
        unsafe {
            if let (Some(dev), Some(hid)) = (USB_BUS.as_mut(), USB_HID.as_mut()) {
                dev.poll(&mut [hid]);
            }
        }

        // check if we need to update the USB
        let mut updating: bool = false;
        if let Some(new_left) = key_left.requires_update() {
            unsafe { KEY_BUFFER[0] = if new_left { key_left.key } else { 0 } };
            updating = true;
        }
        if let Some(new_right) = key_right.requires_update() {
            unsafe { KEY_BUFFER[1] = if new_right { key_right.key } else { 0 } };
            updating = true;
        }

        unsafe {
            if updating {
                push_keyboard_report(KeyboardReport {
                    keycodes: KEY_BUFFER,
                    leds: 0,
                    modifier: 0,
                    reserved: 0,
                })
                .ok();

                // wait at least 5 ms before reporting again
                delay.delay_ms(5u8);
            }
        }
    }
}

fn push_keyboard_report(report: KeyboardReport) -> Result<usize, usb_device::UsbError> {
    free(|_| unsafe { USB_HID.as_mut().map(|h| h.push_input(&report)) }).unwrap()
}
