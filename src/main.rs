#![deny(warnings)]
#![deny(unsafe_code)]
#![no_main]
#![no_std]

extern crate panic_itm;

use cortex_m;
use cortex_m_rt::entry;
use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::{pac, prelude::*};

use cortex_m_log::println;
use cortex_m_log::{
    destination::Itm, printer::itm::InterruptSync as InterruptSyncItm,
};


#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let mut log = InterruptSyncItm::new(Itm::new(cp.ITM));

    // Constrain and Freeze power
    println!(log, "Setup PWR...                  ");
    let pwr = dp.PWR.constrain();
    let vos = pwr.freeze();

    // Constrain and Freeze clock
    println!(log, "Setup RCC...                  ");
    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(400.mhz()).freeze(vos, &dp.SYSCFG);

    println!(log, "");
    println!(log, "stm32h7xx-hal example - Random Blinky");
    println!(log, "");

    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);

    // Configure PE1 as output.
//    let mut led1 = gpioc.pc6.into_push_pull_output();
    let mut led2 = gpioc.pc7.into_push_pull_output();
  //  let mut led3 = gpioc.pc8.into_push_pull_output();

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    loop {
        loop {
    //        led1.set_high().unwrap();
            led2.set_high().unwrap();
      //      led3.set_high().unwrap();
            delay.delay_ms(500_u16);

        //    led1.set_low().unwrap();
            led2.set_low().unwrap();
          //  led3.set_low().unwrap();
            delay.delay_ms(500_u16);
        }
    }
}
