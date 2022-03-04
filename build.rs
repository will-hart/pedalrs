/// Copies the correct configuration files over prior to building based on the target MCU
use std::env;
use std::fs;

fn main() {
    fs::copy(get_memory_name().unwrap(), "memory.x").unwrap();
    fs::copy(get_config_name().unwrap(), ".cargo/config").unwrap();
}

fn get_memory_name() -> std::result::Result<String, String> {
    match env::var("CARGO_FEATURE_STM32F1") {
        Ok(_) => return Ok("memory_stm32f103.x".into()),
        Err(_) => {}
    }
    match env::var("CARGO_FEATURE_STM32F0") {
        Ok(_) => return Ok("memory_stm32f042.x".into()),
        Err(_) => {}
    }

    Err("Uknown chip, specify stm32f1 or stm32f0 as a feature".into())
}

fn get_config_name() -> std::result::Result<String, String> {
    match env::var("CARGO_FEATURE_STM32F1") {
        Ok(_) => return Ok(".cargo/config_stm32f103".into()),
        Err(_) => {}
    }
    match env::var("CARGO_FEATURE_STM32F0") {
        Ok(_) => return Ok(".cargo/config_stm32f042".into()),
        Err(_) => {}
    }

    Err("Uknown chip, specify stm32f1 or stm32f0 as a feature".into())
}
