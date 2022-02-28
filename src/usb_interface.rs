use stm32f1xx_hal::usb::{Peripheral, UsbBus};
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
    UsbError,
};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::hid_class::HIDClass;

use crate::{stateful_key::StatefulKey, usb_descriptor::CustomKeyboardReport};

pub struct UsbInterface<'a> {
    pub hid: HIDClass<'a, UsbBus<Peripheral>>,
    pub bus: UsbDevice<'a, UsbBus<Peripheral>>,
    keyboard_report: CustomKeyboardReport,
    dirty: bool,
}

impl<'a> UsbInterface<'a> {
    /**
     * Creates a new UsbInterface, configures it and returns it
     */
    pub fn new(alloc: &'a UsbBusAllocator<UsbBus<Peripheral>>) -> UsbInterface<'a> {
        // Create the pedal peripheral
        let hid = HIDClass::new(&alloc, CustomKeyboardReport::desc(), 10);

        // TODO: this is a test code from pid.codes, change before release
        let bus = UsbDeviceBuilder::new(&alloc, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Atto Zepto")
            .product("Pedalrs")
            .serial_number("000001")
            .device_release(0x0020)
            .build();

        UsbInterface {
            hid,
            bus,
            keyboard_report: CustomKeyboardReport {
                modifier: 0,
                reserved: 0,
                leds: 0,
                keycodes: [0; 6],
                command: 0,
                data: 0,
            },
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
     * Reads received data from the USB device
     */
    pub fn read_command(&mut self) -> Result<([u8; 64], usize), UsbError> {
        let mut buffer: [u8; 64] = [0; 64];
        match self.hid.pull_raw_output(&mut buffer) {
            Ok(size) => Ok((buffer, size)),
            Err(UsbError::WouldBlock) => {
                // no pending data
                Ok((buffer, 0))
            }
            Err(err) => panic!("Error receiving data {:?}", err),
        }
    }

    /**
     * Sets the relevant pedal state in the USB buffer
     */
    pub fn update_key(&mut self, key: &mut StatefulKey, index: u8) {
        if let Some(updated) = key.requires_update() {
            self.dirty = true;
            self.keyboard_report.keycodes[index as usize] =
                if updated { key.get_code() } else { 0 };
        }
    }

    /**
     * Sends the report, if one is ready to go.
     */
    pub fn send_report(&mut self) -> Result<bool, usb_device::UsbError> {
        if self.dirty {
            self.dirty = false;
            cortex_m::interrupt::free(|_| self.hid.push_input(&self.keyboard_report)).ok();

            return Ok(true);
        }

        Ok(false)
    }
}
