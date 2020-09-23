use arduino_leonardo as hal;
use avr_device::interrupt;
use core::sync::atomic::{AtomicBool, Ordering::SeqCst};
use hal::atmega32u4::USB_DEVICE;

static mut USB: Option<USB_DEVICE> = None;
// static mut USB_CONFIGURATION: AtomicU8 = AtomicU8::new(0);
// static mut CDC_LINE_CODING: [u8; 7] = [0, 255, 0, 0, 0, 0, 8];
// static mut CDC_LINE_RTSDTR: AtomicU8 = AtomicU8::new(0);

static mut CONFIG_STATUS: AtomicBool = AtomicBool::new(false);

pub fn get_cfg_status() -> bool {
    unsafe { CONFIG_STATUS.load(SeqCst) }
}

#[avr_device::interrupt(atmega32u4)]
unsafe fn USB_GEN() {
    interrupt::free(|_| {
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
    });
}

#[avr_device::interrupt(atmega32u4)]
unsafe fn USB_COM() {
    // let usb = get_usb();
    // usb.uenum.write(|w| w.bits(0));
    // let intbits = usb.ueintx.read();
    // if intbits.rxstpi().bit_is_set() {
    //     let bm_request_type = usb.uedatx.read().bits();
    //     let b_request = usb.uedatx.read().bits();
    //     let mut w_value = usb.uedatx.read().bits() as u16;
    //     w_value |= (usb.uedatx.read().bits() as u16) << 8;
    //     let mut w_index = usb.uedatx.read().bits() as u16;
    //     w_index |= (usb.uedatx.read().bits() as u16) << 8;
    //     let mut w_length = usb.uedatx.read().bits() as u16;
    //     w_length |= (usb.uedatx.read().bits() as u16) << 8;
    //     usb.ueintx.write(|w| w.rxstpi().clear_bit());
    //     usb.ueintx.write(|w| w.rxouti().clear_bit());
    //     usb.ueintx.write(|w| w.txini().clear_bit());
    //     // if b_request == 6 {
    //     //     // get descriptor
    //     //     for i in list {
    //     //         if
    //     //     }
    //     // }
    // }
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
        usb.udien.write(|w| w.eorste().set_bit().sofe().set_bit()); // enable end of reset and start of frame interrupts

        unsafe {
            USB = Some(usb); // save peripheral in global
        }
    });

    unsafe {
        interrupt::enable();
    }
}
