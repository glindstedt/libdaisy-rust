//! examples/oled.rs
#![no_main]
#![no_std]
use log::info;
// Includes a panic handler and optional logging facilities
use libdaisy_rust::{CLOCK_RATE_HZ, gpio::{Daisy9, Daisy7}, logger};

use stm32h7xx_hal::{delay::DelayFromTimer, spi};
use stm32h7xx_hal::stm32;
use rtic::cyccnt::{Instant, U32Ext};

use micromath::F32Ext;

use libdaisy_rust::gpio;
use libdaisy_rust::prelude::*;
use libdaisy_rust::system;

// use ssd1306::prelude::*;
use ssd1309::prelude::*;
use embedded_graphics::{fonts::Font6x8, fonts::Text, pixelcolor::BinaryColor, style::TextStyle, prelude::*, primitives::{Line, Circle}, style::{PrimitiveStyle}};

const MILLISECOND: u32 = CLOCK_RATE_HZ.0/1000;
const BLINK_RATE: u32 = 500 * MILLISECOND;
const DRAW_RATE: u32 = MILLISECOND; // Drive Duty = 1/64, so we want to be faster than 15ms, seems like FPS tops out somewhere between 1-15ms
const UPDATE_RATE: u32 = MILLISECOND;

trait F32Tup2Ext {
    fn into_i32(self) -> (i32, i32);
}

impl F32Tup2Ext for (f32, f32) {
    fn into_i32(self) -> (i32, i32) {
        (self.0 as i32, self.1 as i32)
    }
}

/// Rotates a point around the center using radians
fn rotate_point(point: (f32, f32), center: (f32, f32), angle: f32) -> (f32, f32) {
    // https://en.wikipedia.org/wiki/Rotation_matrix#In_two_dimensions
    let x = point.0 - center.0;
    let y = point.1 - center.1;
    (
        center.0 + (x * angle.cos() - y * angle.sin()),
        center.1 + (x * angle.sin() + y * angle.cos()),
    )
}

/// Create an equilateral triangle
///
/// length is the length of the sides
/// angle is the rotation in radians
fn triangle(x: f32, y: f32, length: f32, angle: f32) -> [Line; 3] {
    let altitude = 3.0.sqrt() / 2.0 * length;
    let apothem = altitude / 3.0;

    let peak = rotate_point((x, y - (apothem * 2.0)), (x, y), angle).into_i32();
    let bottom_right = rotate_point((x + (length / 2.0), y + apothem), (x, y), angle).into_i32();
    let bottom_left = rotate_point((x - (length / 2.0), y + apothem), (x, y), angle).into_i32();

    [
        Line::new(bottom_left.into(), peak.into()),
        Line::new(bottom_left.into(), bottom_right.into()),
        Line::new(peak.into(), bottom_right.into()),
    ]
}

/// Create a square
///
/// length is the length of the sides
/// angle is the rotation in radians
fn square(x: f32, y: f32, length: f32, angle: f32) -> [Line; 4] {
    let d = length / 2.0; // distance from center to side

    let top_left = rotate_point((x-d, y-d), (x, y), angle).into_i32();
    let top_right = rotate_point((x+d, y-d), (x, y), angle).into_i32();
    let bottom_left = rotate_point((x-d, y+d), (x, y), angle).into_i32();
    let bottom_right = rotate_point((x+d, y+d), (x, y), angle).into_i32();

    [
        Line::new(top_left.into(), top_right.into()),
        Line::new(top_left.into(), bottom_left.into()),
        Line::new(top_right.into(), bottom_right.into()),
        Line::new(bottom_left.into(), bottom_right.into()),
    ]
}

pub struct State {
    triangle_x: f32,
    triangle_y: f32,
    angle_degrees: f32,
}

impl State {
    pub fn angle_radians(&self) -> f32 {
        self.angle_degrees * core::f32::consts::PI / 180.0
    }

    pub fn update(&mut self, delta: f32) {
        self.triangle_x = (self.triangle_x + delta) % 128.0;
        self.triangle_y = (self.triangle_y + delta) % 64.0;
        self.angle_degrees = (self.angle_degrees + 5.0*delta) % 360.0;
    }
}

const DEFAULT_STATE: State = State { triangle_x: 8.0, triangle_y: 16.0, angle_degrees: 0.0 };

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: gpio::SeedLed,
        display: GraphicsMode<SpiInterface<spi::Spi<stm32::SPI1, spi::Enabled>,Daisy9<Output<PushPull>>,Daisy7<Output<PushPull>>>>,

        #[init(true)]
        led_on: bool,
        #[init(DEFAULT_STATE)]
        state: State,
    }

    #[init(schedule = [blink, draw, update])]
    fn init(ctx: init::Context) -> init::LateResources {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);
        info!("Startup done!");

        let spi1 = system.spi1;

        let cs = system.gpio.daisy7.unwrap().into_push_pull_output();
        let dc = system.gpio.daisy9.unwrap().into_push_pull_output();
        let mut rst = system.gpio.daisy30.unwrap().into_push_pull_output();

        let mut display: GraphicsMode<_> = ssd1309::Builder::new().connect_spi(spi1, dc, cs).into();

        let mut delay = DelayFromTimer::new(system.timer2);
        info!("testing delay, sleeping for 5s ...");
        delay.delay_ms(5000u64);
        info!("5s are up!");
        display.reset(&mut rst, &mut delay).expect("error resetting display");
        display.init().expect("error initializing display");
        display.flush().expect("error flushing display");

        info!("Initialized display");

        let now = ctx.start;
        ctx.schedule.blink(now + BLINK_RATE.cycles()).unwrap();
        ctx.schedule.update(now + UPDATE_RATE.cycles()).unwrap();
        ctx.schedule.draw(now + DRAW_RATE.cycles()).unwrap();

        init::LateResources {
            seed_led: system.gpio.led,
            display: display,
        }
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(schedule = [blink], resources = [seed_led, led_on])]
    fn blink(ctx: blink::Context) {
        if *ctx.resources.led_on {
            ctx.resources.seed_led.set_high().unwrap();
        } else {
            ctx.resources.seed_led.set_low().unwrap();
        }
        *ctx.resources.led_on = !(*ctx.resources.led_on);

        let now = Instant::now();
        ctx.schedule.blink(now + BLINK_RATE.cycles()).unwrap();
    }

    #[task(schedule = [update], resources = [state])]
    fn update(ctx: update::Context) {
        ctx.resources.state.update(1.0);

        let now = Instant::now();
        ctx.schedule.update(now + UPDATE_RATE.cycles()).unwrap();
    }

    #[task(schedule = [draw], resources = [state, display])]
    fn draw(ctx: draw::Context) {
        let state: &State = ctx.resources.state;
        let display = ctx.resources.display;

        let on_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

        display.clear();
        triangle(state.triangle_x, state.triangle_y, 16.0, state.angle_radians()).iter()
            .for_each(|line| {
                line.into_styled(on_style)
                    .draw(display)
                    .expect("error drawing to display");
            });

        square(56.0, 24.0, 24.0, state.angle_radians()).iter()
            .for_each(|line| {
                line.into_styled(on_style)
                    .draw(display)
                    .expect("error drawing to display");
            });

        Circle::new(Point::new(96, 16 + 8), (16.0 * (1.0 + state.angle_radians().sin())) as u32)
            .into_styled(on_style)
            .draw(display)
            .expect("error drawing to display");

        Text::new("Hello World!", Point::new(32, 48))
            .into_styled(TextStyle::new(Font6x8, BinaryColor::On))
            .draw(display)
            .expect("error drawing to display");

        display.flush().expect("error flushing display");

        let now = Instant::now();
        ctx.schedule.draw(now + DRAW_RATE.cycles()).unwrap();
    }

    // RTIC requires that unused interrupts are declared in an extern block when
    // using software tasks; these free interrupts will be used to dispatch the
    // software tasks.
    // https://rtic.rs/0.5/book/en/by-example/tasks.html
    // Check §19.1.2 in the stm32h750 manual to look for unused interrupts
    extern "C" {
        fn JPEG();
    }
};
