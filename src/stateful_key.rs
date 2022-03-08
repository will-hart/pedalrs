use crate::configure::PinType;
use switch_hal::{ActiveLow, InputSwitch, Switch};

pub struct StatefulKey {
    key: u8,
    is_pressed: bool,
    pin: Switch<PinType, ActiveLow>,
}

impl StatefulKey {
    pub fn new(pin: Switch<PinType, ActiveLow>, key: u8) -> StatefulKey {
        StatefulKey {
            key,
            is_pressed: false,
            pin,
        }
    }

    /// Updates the key code associated with this key
    pub fn replace_keycode(&mut self, keycode: u8) {
        self.key = keycode;
    }

    /// Returns the key code associated with this key
    pub fn get_code(&self) -> u8 {
        self.key
    }

    /// Updates the state of the button based on the current state (is_pressed).
    /// Returns Some(bool) if the value has changed and requires a new USB message to be sent,
    /// otherwise returns None
    pub fn requires_update(&mut self) -> Option<bool> {
        match self.pin.is_active() {
            Ok(currently_pressed) => {
                if currently_pressed == self.is_pressed {
                    None
                } else {
                    self.is_pressed = currently_pressed;
                    Some(self.is_pressed)
                }
            }
            Err(_) => panic!("Unable to read button"),
        }
    }
}
