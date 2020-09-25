use arduino_leonardo as hal;
use avr_device::interrupt; // clear flags to handshake
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering::SeqCst};
use hal::atmega32u4::USB_DEVICE;

static mut USB: Option<USB_DEVICE> = None;
static mut USB_CONFIGURATION: AtomicU8 = AtomicU8::new(0); // non-zero when enumerated

// static mut CDC_LINE_CODING: [u8; 7] = [0, 255, 0, 0, 0, 0, 8];
// static mut CDC_LINE_RTSDTR: AtomicU8 = AtomicU8::new(0);

static mut CONFIG_STATUS: AtomicBool = AtomicBool::new(false);

static ENDPOINT_CONFIG_TABLE: [u8; 10] =
    [0x00, 0x01, 0xC1, 0x12, 0x01, 0x80, 0x36, 0x01, 0x81, 0x36];

pub fn get_cfg_status() -> bool {
    unsafe { CONFIG_STATUS.load(SeqCst) }
}

unsafe fn usb_ack_out(usb: &USB_DEVICE) {
    usb.ueintx.write(|w| w.bits(0xFF).rxouti().clear_bit());
}

unsafe fn usb_send_in(usb: &USB_DEVICE) {
    usb.ueintx.write(|w| w.bits(0xFF).txini().clear_bit());
}

unsafe fn usb_wait_in_ready(usb: &USB_DEVICE) {
    while usb.ueintx.read().txini().bit_is_clear() {}
}

unsafe fn usb_wait_receive_out(usb: &USB_DEVICE) {
    while usb.ueintx.read().rxouti().bit_is_clear() {}
}

#[avr_device::interrupt(atmega32u4)]
unsafe fn USB_GEN() {
    let usb = get_usb();
    // end of reset interrupt
    if usb.udint.read().eorsti().bit_is_set() {
        usb.udint.write(|w| w.eorsti().clear_bit()); // clear interrupt flag
        usb.uenum.write(|w| w.bits(0)); // select ep 0
        usb.ueconx.write(|w| w.epen().set_bit()); // enable selected ep
        usb.uecfg0x
            .write(|w| w.eptype().bits(0).epdir().clear_bit()); // set ep to control type an out direction
                                                                // set ep size to 16 bytes, use single bank, and allocate ep
        usb.uecfg1x
            .write(|w| w.epsize().bits(1).epbk().bits(0).alloc().set_bit()); // set ep size to 16 bytes

        usb.ueienx.write(|w| w.rxstpe().set_bit()); // interrupt on setup packet
        CONFIG_STATUS.store(usb.uesta0x.read().cfgok().bit_is_set(), SeqCst);
    }
}

#[avr_device::interrupt(atmega32u4)]
unsafe fn USB_COM() {
    let usb = get_usb();
    usb.uenum.write(|w| w.bits(0)); // select ep 0

    if usb.ueintx.read().rxstpi().bit_is_clear() {
        usb.ueconx.write(|w| w.stallrq().set_bit().epen().set_bit()); // stall
        return;
    }

    let bm_request_type = usb.uedatx.read().bits();
    let b_request = usb.uedatx.read().bits();
    let mut w_value = usb.uedatx.read().bits() as u16;
    w_value |= (usb.uedatx.read().bits() as u16) << 8;
    let mut w_index = usb.uedatx.read().bits() as u16;
    w_index |= (usb.uedatx.read().bits() as u16) << 8;
    let mut w_length = usb.uedatx.read().bits() as u16;
    w_length |= (usb.uedatx.read().bits() as u16) << 8;
    usb.ueintx.write(|w| {
        w.bits(0xFF)
            .rxstpi()
            .clear_bit()
            .rxouti()
            .clear_bit()
            .txini()
            .clear_bit()
    }); // clear flags to handshake

    match b_request {
        0x00 => {
            // get status
            usb_wait_in_ready(&usb);
            usb.uedatx.write(|w| w.bits(0));
            usb.uedatx.write(|w| w.bits(0));
            usb_send_in(&usb);
        }
        0x05 => {
            // set address
            usb_send_in(&usb);
            usb_wait_in_ready(&usb);
            usb.udaddr
                .write(|w| w.bits(w_value as u8).adden().set_bit());
        }
        0x06 => (), // get descriptor
        0x08 => {
            if bm_request_type == 0x80 {
                // get configuration
                usb_wait_in_ready(&usb);
                usb.uedatx.write(|w| w.bits(USB_CONFIGURATION.load(SeqCst)));
                usb_send_in(&usb);
            }
        }
        0x09 => {
            if bm_request_type == 0x00 {
                // set configuration
                USB_CONFIGURATION.store(w_value as u8, SeqCst);
                // cdc_line_rtsdtr = 0
                // transmit_flush_timer = 0
                usb_send_in(&usb);
                let mut cfg = ENDPOINT_CONFIG_TABLE.iter();
                for i in 1..5 {
                    usb.uenum.write(|w| w.bits(i));
                    let en = *cfg.next().unwrap();
                    usb.ueconx.write(|w| w.bits(en));
                    if en != 0 {
                        usb.uecfg0x.write(|w| w.bits(*cfg.next().unwrap()));
                        usb.uecfg1x.write(|w| w.bits(*cfg.next().unwrap()));
                    }
                }
                usb.uerst.write(|w| w.bits(0x1E));
                usb.uerst.write(|w| w.bits(0));
            }
        }
        0x20 => if bm_request_type == 0x21 {}, // set cdc line coding
        0x21 => if bm_request_type == 0xA1 {}, // get cdc line coding
        0x22 => if bm_request_type == 0x21 {}, // set cdc control line state
        _ => (),
    };
}

unsafe fn get_usb() -> &'static USB_DEVICE {
    USB.as_ref().unwrap()
}

pub fn usb_init(usb: USB_DEVICE, pll: &hal::atmega32u4::PLL) {
    interrupt::free(|_| {
        usb.uhwcon.write(|w| w.uvrege().set_bit()); // enable pad regulator
        usb.usbcon.write(|w| w.usbe().set_bit().frzclk().set_bit()); // freeze usb clock
        pll.pllcsr.write(|w| w.pindiv().set_bit().plle().set_bit()); // enable PLL for 16 mhz clock
        while pll.pllcsr.read().plock().bit_is_clear() {} // wait for PLL lock
        usb.usbcon
            .write(|w| w.usbe().set_bit().frzclk().clear_bit().otgpade().set_bit()); // enable usb and vusb
        usb.udcon.write(|w| w.detach().clear_bit()); // enable pull up resistors
        usb.udien.write(|w| w.eorste().set_bit().sofe().set_bit()); // enable end of reset interrupts

        unsafe {
            USB = Some(usb); // save peripheral in global
        }
    });

    unsafe {
        interrupt::enable();
    }
}
