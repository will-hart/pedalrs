use stm32f0xx_hal::usb::{Peripheral, UsbBus};
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::hid_class::HIDClass;

use crate::stateful_key::StatefulKey;

pub struct UsbInterface<'a> {
    pub hid: HIDClass<'a, UsbBus<Peripheral>>,
    pub bus: UsbDevice<'a, UsbBus<Peripheral>>,
    buffer: [u8; 6],
    dirty: bool,
}

impl<'a> UsbInterface<'a> {
    /**
     * Creates a new UsbInterface, configures it and returns it
     */
    pub fn new(alloc: &'a UsbBusAllocator<UsbBus<Peripheral>>) -> UsbInterface<'a> {
        let hid = HIDClass::new(&alloc, KeyboardReport::desc(), 63);

        let bus = UsbDeviceBuilder::new(&alloc, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Atto Zepto")
            .product("Pedalrs")
            .serial_number("000001")
            .device_class(0x03) // HID device from usb.org device classes
            .build();

        UsbInterface {
            hid,
            bus,
            buffer: [0, 0, 0, 0, 0, 0],
            dirty: false,
        }
    }

    /**
     * Polls the USB device
     */
    pub fn poll(&mut self) -> bool {
        self.bus.poll(&mut [&mut self.hid])
    }

    /**
     * Sets the relevant pedal state in the USB buffer
     */
    pub fn update_key(&mut self, key: &mut StatefulKey, index: u8) {
        if let Some(updated) = key.requires_update() {
            self.dirty = true;
            self.buffer[index as usize] = if updated { key.key } else { 0 };
        }
    }

    /**
     * Sends the report, if one is ready to go.
     */
    pub fn send_report(&mut self) -> Result<bool, usb_device::UsbError> {
        if self.dirty {
            self.dirty = false;
            self.push_report(KeyboardReport {
                keycodes: self.buffer,
                leds: 0,
                modifier: 0,
                reserved: 0,
            })
            .ok();

            return Ok(true);
        }

        Ok(false)
    }

    /**
     * Sends the report via HID
     */
    fn push_report(&mut self, report: KeyboardReport) -> Result<usize, usb_device::UsbError> {
        cortex_m::interrupt::free(|_| self.hid.push_input(&report))
    }
}
