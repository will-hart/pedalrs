/** In theory this should be the only place where MCU specific changes are required,
 * for example for moving between the STM32F103 and the STM32F042
 */
use stm32f1xx_hal as hal;

use cortex_m::{asm::delay as cycle_delay, peripheral::NVIC};

use hal::{
    gpio::{ErasedPin, Input, PullUp},
    pac::{self, interrupt::TIM2 as IRQ_TIM2, TIM2},
    prelude::*,
    time::Hertz,
    timer::{CounterHz, Event, SysDelay},
    usb::Peripheral,
};
use switch_hal::{ActiveLow, IntoSwitch, Switch};

pub type PinType = ErasedPin<Input<PullUp>>;

pub struct GpioConfiguration {
    pub btn_left: Switch<PinType, ActiveLow>,
    pub btn_right: Switch<PinType, ActiveLow>,
    pub delay: SysDelay,
    pub peripheral: Peripheral,
    pub timer: CounterHz<TIM2>,
}

pub fn configure_gpio() -> Option<GpioConfiguration> {
    // Get access to device and core peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    // Get access to RCC, AFIO and GPIOA
    let rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    // Set up sysclk and freeze it
    let clocks = rcc
        .cfgr
        .use_hse(Hertz::MHz(8))
        .sysclk(Hertz::MHz(48))
        .pclk1(Hertz::MHz(24))
        .freeze(&mut flash.acr);
    assert!(clocks.usbclk_valid());

    // Set up systick delay
    let delay = cp.SYST.delay(&clocks);

    // Set up GPIO
    let mut gpioa = dp.GPIOA.split();

    // Set up "button" pins
    let btn_left = gpioa
        .pa9
        .into_pull_up_input(&mut gpioa.crh)
        .erase()
        .into_active_low_switch();

    let btn_right = gpioa
        .pa10
        .into_pull_up_input(&mut gpioa.crh)
        .erase()
        .into_active_low_switch();

    // setup the timer for periodic USB report updates
    let mut timer = dp.TIM2.counter_hz(&clocks);
    timer.start(Hertz::Hz(5)).ok();
    timer.listen(Event::Update);
    unsafe {
        cortex_m::interrupt::free(|_| {
            NVIC::unmask(IRQ_TIM2);
            cp.NVIC.set_priority(IRQ_TIM2, 3); // relatively low priority
        });
    }

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    // This forced reset is needed only for development, without it host
    // will not reset your device when you upload new firmware.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
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
        timer,
    });
}
