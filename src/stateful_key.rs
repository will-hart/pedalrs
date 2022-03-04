#[cfg(feature = "stm32f0")]
use stm32f0xx_hal::gpio::{Input, Pin, PullUp};

#[cfg(feature = "stm32f1")]
use stm32f1xx_hal::gpio::{Input, PullUp, Pxx};

use switch_hal::{ActiveLow, InputSwitch, Switch};

#[cfg(feature = "stm32f0")]
pub type PinType<MODE> = Pin<MODE>;

#[cfg(feature = "stm32f1")]
pub type PinType<MODE> = Pxx<MODE>;

pub struct StatefulKey {
    key: u8,
    is_pressed: bool,
    pin: Switch<PinType<Input<PullUp>>, ActiveLow>,
}

impl StatefulKey {
    pub fn new(pin: Switch<PinType<Input<PullUp>>, ActiveLow>, key: u8) -> StatefulKey {
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
