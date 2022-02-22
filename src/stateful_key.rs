use stm32f0xx_hal::gpio::{Input, Pin, PullUp};
use switch_hal::{ActiveLow, InputSwitch, Switch};

pub struct StatefulKey {
    pub key: u8,
    is_pressed: bool,
    pin: Switch<Pin<Input<PullUp>>, ActiveLow>,
}

impl StatefulKey {
    pub fn new(pin: Switch<Pin<Input<PullUp>>, ActiveLow>, key: u8) -> StatefulKey {
        StatefulKey {
            key,
            is_pressed: false,
            pin,
        }
    }

    // Updates the state of the button based on the current state (is_pressed).
    // Returns Some(bool) if the value has changed and requires a new USB message to be sent,
    // otherwise returns None
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
