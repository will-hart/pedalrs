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

    /// Updates the state of the button based on the current state (is_pressed).
    pub fn update(&mut self) {
        match self.pin.is_active() {
            Ok(currently_pressed) => {
                self.is_pressed = currently_pressed;
            }
            Err(_) => panic!("Unable to read button"),
        }
    }

    /// Returns the current keycode -
    /// either self.get_code() if the key is pressed or 0 if the key isn't pressed
    pub fn current_keycode(&self) -> u8 {
        if self.is_pressed {
            self.key
        } else {
            0
        }
    }
}
