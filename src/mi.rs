use crate::io_ptr;

pub const MI_BASE: u32 = 0x0430_0000;

pub const MODE: u32 = MI_BASE + 0x00;
pub const VERSION: u32 = MI_BASE + 0x04;
pub const INTERRUPT: u32 = MI_BASE + 0x08;
pub const MASK: u32 = MI_BASE + 0x0C;

pub const BB_SECURE_EXCEPTION: u32 = MI_BASE + 0x14;

pub const BB_RANDOM: u32 = MI_BASE + 0x2C;

pub const BB_INTERRUPT: u32 = MI_BASE + 0x38;
pub const BB_MASK: u32 = MI_BASE + 0x3C;

const MI_MODE: *mut u32 = io_ptr!(mut MODE);
const MI_VERSION: *mut u32 = io_ptr!(mut VERSION);
const MI_INTERRUPT: *mut u32 = io_ptr!(mut INTERRUPT);
const MI_MASK: *mut u32 = io_ptr!(mut MASK);

const MI_BB_SECURE_EXCEPTION: *mut u32 = io_ptr!(mut BB_SECURE_EXCEPTION);

const MI_BB_RANDOM: *mut u32 = io_ptr!(mut BB_RANDOM);

const MI_BB_INTERRUPT: *mut u32 = io_ptr!(mut BB_INTERRUPT);
const MI_BB_MASK: *mut u32 = io_ptr!(mut BB_MASK);

pub struct Mi;

impl Mi {
    const fn new() -> Self {
        Self {}
    }

    pub fn mode(&self) -> u32 {
        unsafe { MI_MODE.read_volatile() }
    }

    pub fn version(&self) -> u32 {
        unsafe { MI_VERSION.read_volatile() }
    }

    pub fn interrupt(&self) -> u32 {
        unsafe { MI_INTERRUPT.read_volatile() }
    }

    pub fn mask(&self) -> u32 {
        unsafe { MI_MASK.read_volatile() }
    }

    pub fn bb_secure_exception(&self) -> u32 {
        unsafe { MI_BB_SECURE_EXCEPTION.read_volatile() }
    }

    pub fn bb_random(&self) -> u32 {
        unsafe { MI_BB_RANDOM.read_volatile() }
    }

    pub fn bb_interrupt(&self) -> u32 {
        unsafe { MI_BB_INTERRUPT.read_volatile() }
    }

    pub fn bb_mask(&self) -> u32 {
        unsafe { MI_BB_MASK.read_volatile() }
    }

    pub fn set_mode(&mut self, val: u32) {
        unsafe { MI_MODE.write_volatile(val) }
    }

    pub fn set_version(&mut self, val: u32) {
        unsafe { MI_VERSION.write_volatile(val) }
    }

    pub fn set_interrupt(&mut self, val: u32) {
        unsafe { MI_INTERRUPT.write_volatile(val) }
    }

    pub fn set_mask(&mut self, val: u32) {
        unsafe { MI_MASK.write_volatile(val) }
    }

    pub fn set_bb_secure_exception(&mut self, val: u32) {
        unsafe { MI_BB_SECURE_EXCEPTION.write_volatile(val) }
    }

    pub fn set_bb_random(&mut self, val: u32) {
        unsafe { MI_BB_RANDOM.write_volatile(val) }
    }

    pub fn set_bb_interrupt(&mut self, val: u32) {
        unsafe { MI_BB_INTERRUPT.write_volatile(val) }
    }

    pub fn set_bb_mask(&mut self, val: u32) {
        unsafe { MI_BB_MASK.write_volatile(val) }
    }
}

static mut MI: Mi = Mi::new();

pub fn mi() -> &'static mut Mi {
    unsafe { &mut MI }
}
