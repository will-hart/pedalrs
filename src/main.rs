#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use hal::{delay::Delay, pac, prelude::*};
use stm32f1xx_hal as hal;
use switch_hal::{InputSwitch, IntoSwitch, OutputSwitch};

#[entry]
fn main() -> ! {
    /* Get access to device and core peripherals */
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    /* Get access to RCC, AFIO and GPIOA */
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    /* Set up "button" pin */
    let btn = gpiob
        .pb5
        .into_pull_up_input(&mut gpiob.crl)
        .into_active_low_switch();

    /* Set up LED pin */
    let mut led = gpioc
        .pc13
        .into_push_pull_output(&mut gpioc.crh)
        .into_active_low_switch();

    /* Set up sysclk and freeze it */
    let clocks = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);

    /* Set up systick delay */
    let mut delay = Delay::new(cp.SYST, clocks);

    /* Light show */
    led.on().ok();
    delay.delay_ms(500_u16);
    led.off().ok();
    delay.delay_ms(500_u16);
    led.on().ok();
    delay.delay_ms(3000_u16);
    led.off().ok();
    delay.delay_ms(3000_u16);

    loop {
        // map the button
        match btn.is_active() {
            Ok(true) => {
                led.on().ok();
            }
            Ok(false) => {
                led.off().ok();
            }
            Err(_) => {
                panic!("Failed to read button state");
            }
        }
    }
}
