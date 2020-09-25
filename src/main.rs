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
    disp.write_str("init ").unwrap();

    usb_serial::init();
    while !usb_serial::is_configured() {}
    avr_device::interrupt::free(|_| disp.write_str("done ").ok());
    while !usb_serial::get_dtr() {}
    usb_serial::flush_input();
    avr_device::interrupt::free(|_| disp.write_str("open\n").ok());

    loop {
        while usb_serial::get_available() > 0 {
            if let Some(c) = usb_serial::get_char() {
                avr_device::interrupt::free(|_| disp.print_char(c as char).ok());
            }
        }

        hal::delay_ms(200);
        led.toggle().void_unwrap();
    }
}

#[avr_device::interrupt(atmega32u4)]
unsafe fn USB_GEN() {
    avr_device::interrupt::free(|_| usb_serial::isr_usb_gen_vect());
}

#[avr_device::interrupt(atmega32u4)]
unsafe fn USB_COM() {
    avr_device::interrupt::free(|_| usb_serial::isr_usb_com_vect());
}
