use embedded_hal::digital::v2::OutputPin;
/** In theory this should be the only place where MCU specific changes are required,
 * for example for moving between the STM32F103 and the STM32F042
 */
use stm32f1xx_hal as hal;

use cortex_m::asm::delay as cycle_delay;
use hal::{
    delay::Delay,
    gpio::{Input, PullUp, Pxx},
    pac::{self},
    prelude::*,
    usb::Peripheral,
};
use switch_hal::{ActiveLow, IntoSwitch, Switch};

pub struct GpioConfiguration {
    pub btn_left: Switch<Pxx<Input<PullUp>>, ActiveLow>,
    pub btn_right: Switch<Pxx<Input<PullUp>>, ActiveLow>,
    pub delay: Delay,
    pub peripheral: Peripheral,
}

pub fn configure_gpio() -> Option<GpioConfiguration> {
    /* Get access to device and core peripherals */
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    /* Get access to RCC, AFIO and GPIOA */
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    /* Set up sysclk and freeze it */
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);
    assert!(clocks.usbclk_valid());

    /* Set up systick delay */
    let delay = Delay::new(cp.SYST, clocks);

    /* Set up GPIO */
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    /* Set up "button" pins */
    let btn_left = gpioa
        .pa9
        .into_pull_up_input(&mut gpioa.crh)
        .downgrade()
        .into_active_low_switch();

    let btn_right = gpioa
        .pa10
        .into_pull_up_input(&mut gpioa.crh)
        .downgrade()
        .into_active_low_switch();

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low().ok();
    cycle_delay(100); // >1 us, I think

    return Some(GpioConfiguration {
        btn_left,
        btn_right,
        delay,
        peripheral: Peripheral {
            usb: dp.USB,
            pin_dm: gpioa.pa11,
            pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
        },
    });
}
