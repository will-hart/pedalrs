#[cfg(feature = "stm32f042")]
mod stm32f042;

#[cfg(feature = "stm32f042")]
pub use stm32f042::{configure_gpio, GpioConfiguration};
