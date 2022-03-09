#[cfg(feature = "stm32f0")]
use stm32f0xx_hal::usb::{Peripheral, UsbBus};
#[cfg(feature = "stm32f1")]
use stm32f1xx_hal::usb::{Peripheral, UsbBus};

use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
    UsbError,
};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::hid_class::HIDClass;

use crate::usb_descriptor::{CommandReport, CustomKeyboardReport};

pub struct UsbInterface<'a> {
    pub hid: HIDClass<'a, UsbBus<Peripheral>>,
    command: HIDClass<'a, UsbBus<Peripheral>>,
    pub bus: UsbDevice<'a, UsbBus<Peripheral>>,
    keyboard_report: CustomKeyboardReport,
}

impl<'a> UsbInterface<'a> {
    /// Creates a new UsbInterface, configures it and returns it
    pub fn new(alloc: &'a UsbBusAllocator<UsbBus<Peripheral>>) -> UsbInterface<'a> {
        // Create the pedal peripheral
        let hid = HIDClass::new(&alloc, CustomKeyboardReport::desc(), 10);
        let command = HIDClass::new_ep_out(&alloc, CommandReport::desc(), 10);

        // TODO: this is a test code from pid.codes, change before release
        let bus = UsbDeviceBuilder::new(&alloc, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Atto Zepto")
            .product("Pedalrs")
            .serial_number("000001")
            .device_release(0x0020)
            .build();

        UsbInterface {
            hid,
            command,
            bus,
            keyboard_report: CustomKeyboardReport {
                modifier: 0,
                reserved: 0,
                leds: 0,
                keycodes: [0; 6],
            },
        }
    }

    /// Polls the USB device
    pub fn poll(&mut self) -> bool {
        self.bus.poll(&mut [&mut self.hid, &mut self.command])
    }

    /// Reads received data from the USB device
    pub fn read_command(&mut self) -> Result<([u8; 64], usize), UsbError> {
        let mut buffer: [u8; 64] = [0; 64];
        match self.command.pull_raw_output(&mut buffer) {
            Ok(size) => Ok((buffer, size)),
            Err(UsbError::WouldBlock) => {
                // no pending data
                Ok((buffer, 0))
            }
            Err(err) => panic!("Error receiving data {:?}", err),
        }
    }

    /// Sends the report, if one is ready to go.
    pub fn send_report(
        &mut self,
        key1: u8,
        key2: u8,
        force_update: bool,
    ) -> Result<bool, usb_device::UsbError> {
        // if either key pressed value has changed, send a report
        if force_update
            || self.keyboard_report.keycodes[0] != key1
            || self.keyboard_report.keycodes[1] != key2
        {
            self.keyboard_report.keycodes[0] = key1;
            self.keyboard_report.keycodes[1] = key2;
            self.hid.push_input(&self.keyboard_report).ok();

            return Ok(true);
        }

        Ok(false)
    }
}
