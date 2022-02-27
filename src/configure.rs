/** In theory this should be the only place where MCU specific changes are required,
 * for example for moving between the STM32F103 and the STM32F042
 */
use stm32f0xx_hal as hal;

use cortex_m::asm::delay as cycle_delay;
use embedded_hal::digital::v2::OutputPin;
use hal::{
    delay::Delay,
    gpio::{gpioc::PC13, Input, Output, Pin, PullUp, PushPull},
    pac::{self},
    prelude::*,
    rcc::HSEBypassMode,
    usb::Peripheral,
};
use switch_hal::{ActiveLow, IntoSwitch, Switch};

pub struct GpioConfiguration {
    pub btn_left: Switch<Pin<Input<PullUp>>, ActiveLow>,
    pub btn_right: Switch<Pin<Input<PullUp>>, ActiveLow>,
    pub led: Option<Switch<PC13<Output<PushPull>>, ActiveLow>>,
    pub peripheral: Peripheral,
    pub delay: Delay,
}

pub fn configure_gpio() -> Option<GpioConfiguration> {
    /* Get access to device and core peripherals */
    let mut dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    /* remap USB pins */
    stm32f0xx_hal::usb::remap_pins(&mut dp.RCC, &mut dp.SYSCFG);

    /* Set up sysclk and freeze it */
    let mut clocks = dp
        .RCC
        .configure()
        .hse(16.mhz(), HSEBypassMode::Bypassed)
        .sysclk(48.mhz())
        .pclk(24.mhz())
        .freeze(&mut dp.FLASH);

    /* Set up systick delay */
    let delay = Delay::new(cp.SYST, &clocks);

    /* Set up GPIO */
    let gpioa = dp.GPIOA.split(&mut clocks);
    let gpiob = dp.GPIOB.split(&mut clocks);

    /* Set up "button" pins */
    // TODO: Get correct pins
    let (btn_left, btn_right, mut usb_dp, usb_dm) = cortex_m::interrupt::free(move |cs| {
        (
            gpioa
                .pa5
                .into_pull_up_input(cs)
                .downgrade()
                .into_active_low_switch(),
            gpiob
                .pb1
                .into_pull_up_input(cs)
                .downgrade()
                .into_active_low_switch(),
            gpioa.pa12.into_push_pull_output(cs),
            gpioa.pa11.into_floating_input(cs),
        )
    });

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    usb_dp.set_low().ok();
    cycle_delay(100); // >1 us, I think

    return Some(GpioConfiguration {
        btn_left,
        btn_right,
        peripheral: Peripheral {
            usb: dp.USB,
            pin_dm: usb_dm,
            pin_dp: cortex_m::interrupt::free(move |cs| usb_dp.into_floating_input(cs)),
        },
        led: None,
        delay,
    });
}
