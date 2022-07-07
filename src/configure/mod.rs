#[cfg(feature = "stm32f1")]
use eeprom::EEPROMExt;

#[cfg(feature = "stm32f0")]
mod stm32f042;
#[cfg(feature = "stm32f0")]
pub use stm32f042::{configure_gpio, GpioConfiguration, PinType, TimerType};

#[cfg(feature = "stm32f1")]
mod stm32f103;
#[cfg(feature = "stm32f1")]
pub use stm32f103::{configure_gpio, GpioConfiguration, PinType, TimerType, EEPROM_PARAMS};
#[cfg(feature = "stm32f1")]
use stm32f1xx_hal::flash::Parts;

#[cfg(feature = "stm32f1")]
pub const EEPROM_PAGES: u32 = 2; // need to reserve these in memory.x

#[cfg(feature = "stm32f1")]
pub static KEYBIND_CONFIG_TAG: u16 = 0x01u16;

#[cfg(feature = "stm32f1")]
pub static SHIFT_WHEN_BOTH_PRESSED_CONFIG_TAG: u16 = 0x02u16;

pub struct Configuration {
    pub left_key: u8,
    pub right_key: u8,
    pub both_for_shift: bool,
}

#[cfg(feature = "stm32f1")]
fn keys_to_bytes(a: u8, b: u8) -> u16 {
    ((a as u16) << 8) & (b as u16)
}

#[cfg(feature = "stm32f1")]
fn bytes_to_keys(data: u16) -> (u8, u8) {
    ((data >> 8) as u8, data as u8)
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            left_key: 0x14u8,
            right_key: 0x08u8,
            both_for_shift: true,
        }
    }
}

impl Configuration {
    /// Reads configuration data from flash and returns a configuration structure
    #[cfg(feature = "stm32f1")]
    pub fn read(flash: &mut Parts) -> Self {
        let mut eeprom = flash.eeprom(EEPROM_PARAMS);
        let keys = bytes_to_keys(eeprom.read(KEYBIND_CONFIG_TAG).unwrap_or(0x1408_u16));
        let is_bool = bytes_to_keys(
            eeprom
                .read(SHIFT_WHEN_BOTH_PRESSED_CONFIG_TAG)
                .unwrap_or(0x01u16),
        );

        Configuration {
            left_key: keys.0,
            right_key: keys.1,
            both_for_shift: is_bool.0 > 0,
        }
    }

    #[cfg(feature = "stm32f1")]
    /// Writes the configuration to flash
    pub fn write(&mut self, flash: &mut Parts) {
        let mut eeprom = flash.eeprom(EEPROM_PARAMS);

        let page1 = keys_to_bytes(self.left_key, self.right_key);
        eeprom
            .write(KEYBIND_CONFIG_TAG, page1)
            .expect("Failed writing page1 to flash");

        let page2 = keys_to_bytes(if self.both_for_shift { 0x01u8 } else { 0u8 }, 0u8);
        eeprom
            .write(SHIFT_WHEN_BOTH_PRESSED_CONFIG_TAG, page2)
            .expect("Failed writing page2 to flash");
    }
}

#[cfg(test)]
mod test {
    use crate::config::{bytes_to_keys, keys_to_bytes};

    #[test]
    fn u8s_encode_to_u16() {
        assert_eq!(keys_to_bytes(0x07u8, 0x08u8), 0x0708u16);
    }

    #[test]
    fn u16_decodes_to_u8s() {
        assert_eq!(bytes_to_keys(0x0708u16), (0x07u8, 0x08u8))
    }
}
