#![no_std]
#![no_main]

use arduino_leonardo::prelude::*;
use panic_halt as _;

#[arduino_leonardo::entry]
fn main() -> ! {
    let dp = arduino_leonardo::Peripherals::take().unwrap();
    let mut pins = arduino_leonardo::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD, dp.PORTE);
    let mut led = pins.d13.into_output(&mut pins.ddr);

    loop {
        led.toggle().void_unwrap();
        arduino_leonardo::delay_ms(500);
    }
}
