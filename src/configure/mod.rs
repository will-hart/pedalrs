#[cfg(feature = "stm32f0")]
mod stm32f042;
#[cfg(feature = "stm32f0")]
pub use stm32f042::{configure_gpio, GpioConfiguration, PinType};

#[cfg(feature = "stm32f1")]
mod stm32f103;
#[cfg(feature = "stm32f1")]
pub use stm32f103::{configure_gpio, GpioConfiguration, PinType};
