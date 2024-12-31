use crate::io_ptr;

macro_rules! USB_BASE {
    ($e:expr) => {
        0x0490_0000u32 + 0x0010_0000u32 * $e
    };
}

#[rustfmt::skip]
macro_rules! usb_reg {
    ($reg:ident, $off:expr) => {
        macro_rules! $reg {
            ($e:expr) => {
                io_ptr!(mut USB_BASE!($e) + $off)
            };
        }
    };
}

usb_reg!(USB_SEC_MODE, 0x40010);

pub struct Usb<const N: u32>;

impl<const N: u32> Usb<N> {
    const fn new() -> Self {
        Self {}
    }

    pub fn sec_mode(&self) -> u32 {
        unsafe { USB_SEC_MODE!(N).read_volatile() }
    }

    pub fn set_sec_mode(&mut self, val: u32) {
        unsafe { USB_SEC_MODE!(N).write_volatile(val) }
    }
}

static mut USB0: Usb<0> = Usb::new();
static mut USB1: Usb<1> = Usb::new();

pub fn usb0() -> &'static mut Usb<0> {
    unsafe { &mut USB0 }
}

pub fn usb1() -> &'static mut Usb<1> {
    unsafe { &mut USB1 }
}