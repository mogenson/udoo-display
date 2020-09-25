#![allow(dead_code)]

extern "C" {
    /* setup */
    fn usb_init();
    fn usb_configured() -> u8;

    /* receiving data */
    fn usb_serial_getchar() -> i16;
    fn usb_serial_available() -> u8;
    fn usb_serial_flush_input();

    /* transmitting data */
    fn usb_serial_putchar(c: u8) -> i8;
    fn usb_serial_putchar_nowait(c: u8) -> i8;
    fn usb_serial_write(buffer: *const u8, size: u16) -> i8;
    fn usb_serial_flush_output();

    /* serial parameters */
    fn usb_serial_get_baud() -> u32;
    fn usb_serial_get_stopbits() -> u8;
    fn usb_serial_get_paritytype() -> u8;
    fn usb_serial_get_numbits() -> u8;
    fn usb_serial_get_control() -> u8;
    fn usb_serial_set_control(signals: u8) -> i8;

    /* interrupt service routines */
    pub fn isr_usb_gen_vect();
    pub fn isr_usb_com_vect();
}

pub fn init() {
    unsafe { usb_init() };
}

pub fn is_configured() -> bool {
    unsafe { usb_configured() != 0 }
}

pub fn get_char() -> Option<u8> {
    let c = unsafe { usb_serial_getchar() };
    if c == -1 {
        None
    } else {
        Some(c as u8)
    }
}

pub fn get_available() -> u8 {
    unsafe { usb_serial_available() }
}

pub fn flush_input() {
    unsafe { usb_serial_flush_input() }
}

pub fn get_dtr() -> bool {
    unsafe { usb_serial_get_control() & 0x01 != 0 }
}

pub fn get_rts() -> bool {
    unsafe { usb_serial_get_control() & 0x02 != 0 }
}
