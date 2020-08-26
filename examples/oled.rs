//! examples/oled.rs
#![no_main]
#![no_std]
use log::info;
// Includes a panic handler and optional logging facilities
use libdaisy_rust::{gpio::{Daisy9, Daisy7}, logger};

use stm32h7xx_hal::{time::Hertz, spi};
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::timer::Timer;

use libdaisy_rust::gpio;
use libdaisy_rust::prelude::*;
use libdaisy_rust::system;

// use ssd1306::prelude::*;
use ssd1309::prelude::*;
use embedded_graphics::{fonts::Font6x8, fonts::Text, pixelcolor::BinaryColor, style::TextStyle, prelude::*, primitives::{Rectangle, Line, Circle}, style::{PrimitiveStyle}};

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: gpio::SeedLed,
        timer2: Timer<stm32::TIM2>,
        display: GraphicsMode<SpiInterface<spi::Spi<stm32::SPI1, spi::Enabled>,Daisy9<Output<PushPull>>,Daisy7<Output<PushPull>>>>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);
        info!("Startup done!");

        // system.timer2.set_freq(Hertz(30));
        system.timer2.set_freq(500.ms());

        let spi1 = system.spi1;

        let cs = system.gpio.daisy7.unwrap().into_push_pull_output();
        let dc = system.gpio.daisy9.unwrap().into_push_pull_output();
        let mut rst = system.gpio.daisy30.unwrap().into_push_pull_output();

        // let mut display: GraphicsMode<_> = ssd1309::Builder::new().connect_spi(spi1, dc, ssd1309::NoOutputPin::new()).into();
        let mut display: GraphicsMode<_> = ssd1309::Builder::new().connect_spi(spi1, dc, cs).into();

        display.reset(&mut rst, &mut system.delay).expect("error resetting display");
        display.init().expect("error initializing display");
        display.flush().expect("error flushing display");

        info!("Initialized display");

        init::LateResources {
            seed_led: system.gpio.led,
            timer2: system.timer2,
            display: display,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task( binds = TIM2, resources = [timer2, seed_led, display] )]
    fn blink(ctx: blink::Context) {
        static mut LED_IS_ON: bool = true;

        ctx.resources.timer2.clear_irq();

        let display = ctx.resources.display;

        let mut color = BinaryColor::On;

        if *LED_IS_ON {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
            color = BinaryColor::Off;
        }
        *LED_IS_ON = !(*LED_IS_ON);

        Line::new(Point::new(8, 16 + 16), Point::new(8 + 16, 16 + 16))
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(display)
            .expect("error drawing to display");

        Line::new(Point::new(8, 16 + 16), Point::new(8 + 8, 16))
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(display)
            .expect("error drawing to display");

        Line::new(Point::new(8 + 16, 16 + 16), Point::new(8 + 8, 16))
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(display)
            .expect("error drawing to display");

        Rectangle::new(Point::new(48, 16), Point::new(48 + 16, 16 + 16))
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(display)
            .expect("error drawing to display");

        Circle::new(Point::new(96, 16 + 8), 8)
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(display)
            .expect("error drawing to display");

        Text::new("Hello World!", Point::new(32, 48))
            .into_styled(TextStyle::new(Font6x8, BinaryColor::On))
            .draw(display)
            .expect("error drawing to display");

        display.flush().expect("error flushing display");

    }
};
