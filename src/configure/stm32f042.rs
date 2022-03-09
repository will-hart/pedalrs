/** In theory this should be the only place where MCU specific changes are required,
 * for example for moving between the STM32F103 and the STM32F042
 */
use stm32f0xx_hal as hal;

use cortex_m::{asm::delay as cycle_delay, peripheral::NVIC};
use embedded_hal::digital::v2::OutputPin;
use hal::{
    delay::Delay,
    gpio::{Input, Pin, PullUp},
    pac::{self, interrupt::TIM2 as IRQ_TIM2, TIM2},
    prelude::*,
    rcc::HSEBypassMode,
    time::Hertz,
    timers::{Event, Timer},
    usb::Peripheral,
};
use switch_hal::{ActiveLow, IntoSwitch, Switch};

pub type PinType = Pin<Input<PullUp>>;
pub type TimerType = Timer<TIM2>;

pub struct GpioConfiguration {
    pub btn_left: Switch<PinType, ActiveLow>,
    pub btn_right: Switch<PinType, ActiveLow>,
    pub delay: Delay,
    pub peripheral: Peripheral,
    pub timer: TimerType,
}

pub fn configure_gpio() -> Option<GpioConfiguration> {
    /* Get access to device and core peripherals */
    let mut dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    /* remap USB pins */
    stm32f0xx_hal::usb::remap_pins(&mut dp.RCC, &mut dp.SYSCFG);

    /* Set up sysclk and freeze it */
    let mut clocks = dp
        .RCC
        .configure()
        .hse(16.mhz(), HSEBypassMode::NotBypassed)
        .sysclk(48.mhz())
        .hclk(48.mhz())
        .pclk(48.mhz())
        .freeze(&mut dp.FLASH);

    /* Set up systick delay */
    let delay = Delay::new(cp.SYST, &clocks);

    /* Set up GPIO */
    let gpioa = dp.GPIOA.split(&mut clocks);
    let gpiob = dp.GPIOB.split(&mut clocks);

    /* Set up "button" pins */
    // TODO: Get correct pins
    let (btn_left, btn_right, mut usb_dp, usb_dm, timer) = cortex_m::interrupt::free(move |cs| {
        // Set up a timer ticking at 5 Hz
        let mut timer = Timer::tim2(dp.TIM2, Hertz(5), &mut clocks);
        timer.start(Hertz(5));
        timer.listen(Event::TimeOut);

        unsafe {
            NVIC::unmask(IRQ_TIM2);
            cp.NVIC.set_priority(IRQ_TIM2, 3);
        }

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
            timer,
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
        delay,
        timer,
    });
}
