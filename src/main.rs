#![no_std]
#![no_main]

use arduino_leonardo::prelude::*;
use atmega32u4_usb_serial::UsbSerial;
use panic_halt as _;
use ssd1306::prelude::*;

#[arduino_leonardo::entry]
fn main() -> ! {
    let dp = arduino_leonardo::Peripherals::take().unwrap();
    let pins = arduino_leonardo::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD, dp.PORTE, dp.PORTF);
    let mut usb = UsbSerial::new(dp.USB_DEVICE);

    let i2c = arduino_leonardo::I2c::new(
        dp.TWI,
        pins.d2.into_pull_up_input(&pins.ddr),
        pins.d3.into_pull_up_input(&pins.ddr),
        400_000,
    );

    let interface = ssd1306::I2CDIBuilder::new().init(i2c);
    let mut disp: TerminalMode<_, _> = ssd1306::Builder::new()
        .size(DisplaySize128x32)
        .connect(interface)
        .into();

    disp.init().unwrap();
    disp.clear().unwrap();

    usb.init(&dp.PLL);

    loop {
        if let Ok(c) = nb::block!(usb.read()) {
            avr_device::interrupt::free(|_| {
                if c == '\0' as u8 {
                    disp.clear()
                } else {
                    disp.print_char(c as char)
                }
                .ok();
            });
        }
    }
}
