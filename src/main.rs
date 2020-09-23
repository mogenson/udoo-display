#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod usb_serial;

use arduino_leonardo as hal;
use core::fmt::Write;
use hal::prelude::*;
use panic_halt as _;
use ssd1306::{prelude::*, Builder, I2CDIBuilder};

#[hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let mut pins = hal::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD, dp.PORTE, dp.PORTF);
    let mut led = pins.d13.into_output(&mut pins.ddr);
    let i2c = hal::I2c::new(
        dp.TWI,
        pins.d2.into_pull_up_input(&mut pins.ddr),
        pins.d3.into_pull_up_input(&mut pins.ddr),
        400_000,
    );

    let interface = I2CDIBuilder::new().init(i2c);
    let mut disp: TerminalMode<_> = Builder::new().connect(interface).into();
    disp.init().unwrap();
    disp.clear().unwrap();
    disp.write_str("USB init\n").unwrap();

    usb_serial::usb_init(dp.USB_DEVICE, &dp.PLL);
    while !usb_serial::get_cfg_status() {
        hal::delay_ms(5);
    }
    disp.write_str("done").unwrap();

    loop {
        hal::delay_ms(1000);
        led.toggle().void_unwrap();
    }
}
