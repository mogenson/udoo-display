# atmega32u4-usb

Display text received over a USB serial interface to an SSD1306 OLED screen, on an ATmega32U4 microcontroller.

Main application logic and display driver is written in Rust. USB serial implimentation uses [Teensy C code](https://www.pjrc.com/teensy/usb_serial.html).
Run `cargo make build` to compile and `cargo make flash` to upload.
