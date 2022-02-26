use serde::ser::{Serialize, SerializeTuple, Serializer};
use usbd_hid::descriptor::{gen_hid_descriptor, AsInputReport, SerializedDescriptor};

/// KeyboardReport describes a report and its companion descriptor that can be
/// used to send keyboard button presses to a host and receive the status of the
/// keyboard LEDs.
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = KEYBOARD) = {
        (usage_page = KEYBOARD, usage_min = 0xE0, usage_max = 0xE7) = {
            #[packed_bits 8] #[item_settings data,variable,absolute] modifier=input;
        };
        (usage_min = 0x00, usage_max = 0xFF) = {
            #[item_settings constant,variable,absolute] reserved=input;
        };
        (usage_page = LEDS, usage_min = 0x01, usage_max = 0x03) = {
            #[item_settings data,variable,absolute] command=output;
        };
        (usage_page = LEDS, usage_min = 0x00, usage_max = 0xFF) = {
            #[item_settings data,variable,absolute] value=output;
        };
        (usage_page = KEYBOARD, usage_min = 0x00, usage_max = 0xDD) = {
            #[item_settings data,array,absolute] keycodes=input;
        };
    }
)]
#[allow(dead_code)]
pub struct CustomKeyboardReport {
    pub modifier: u8,
    pub reserved: u8,
    pub command: u8,
    pub value: u8,
    pub keycodes: [u8; 6],
}
