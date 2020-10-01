# atmega32u4-usb

Display text received over a USB serial interface to an SSD1306 OLED screen, on an ATmega32U4 microcontroller.

Main application logic and display driver is written in Rust. USB serial implimentation uses [Teensy C code](https://www.pjrc.com/teensy/usb_serial.html).

Run `cargo make build` to compile and `cargo make flash` to upload.

Write data to serial port to display on the screen. For example: `ip addr | grep -Po '(?<=inet)\s(?!127)[^/]+' > /dev/ttyACM0`, to show the current IP address. Write a null character to clear screen: `echo -ne '\0' > /dev/ttyACM0`.
