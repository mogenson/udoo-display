#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod usb_serial;

use arduino_leonardo::prelude::*;
use panic_halt as _;
use ssd1306::prelude::*;

#[arduino_leonardo::entry]
fn main() -> ! {
    let dp = arduino_leonardo::Peripherals::take().unwrap();
    let pins = arduino_leonardo::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD, dp.PORTE, dp.PORTF);
    let mut usb = usb_serial::UsbSerial();

    let i2c = arduino_leonardo::I2c::new(
        dp.TWI,
        pins.d2.into_pull_up_input(&pins.ddr),
        pins.d3.into_pull_up_input(&pins.ddr),
        400_000,
    );

    let interface = ssd1306::I2CDIBuilder::new().init(i2c);
    let mut disp: TerminalMode<_> = ssd1306::Builder::new().connect(interface).into();
    disp.init().unwrap();
    disp.clear().unwrap();

    usb.init();
    while !usb.is_configured() {}

    loop {
        match usb.read() {
            Ok(0) => avr_device::interrupt::free(|_| disp.clear().ok()),
            Ok(c) => avr_device::interrupt::free(|_| disp.print_char(c as char).ok()),
            Err(_) => Some(()),
        };
    }
}

#[avr_device::interrupt(atmega32u4)]
unsafe fn USB_GEN() {
    avr_device::interrupt::free(|_| usb_serial::usb_gen_handler());
}

#[avr_device::interrupt(atmega32u4)]
unsafe fn USB_COM() {
    avr_device::interrupt::free(|_| usb_serial::usb_com_handler());
}
