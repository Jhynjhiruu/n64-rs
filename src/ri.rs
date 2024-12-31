use crate::io_ptr;

const RI_BASE: u32 = 0x0470_0000;

const RI_MODE: *mut u32 = io_ptr!(mut RI_BASE + 0x00);
const RI_CONFIG: *mut u32 = io_ptr!(mut RI_BASE + 0x04);
const RI_CURRENT_LOAD: *mut u32 = io_ptr!(mut RI_BASE + 0x08);
const RI_SELECT: *mut u32 = io_ptr!(mut RI_BASE + 0x0C);
const RI_REFRESH: *mut u32 = io_ptr!(mut RI_BASE + 0x10);
const RI_LATENCY: *mut u32 = io_ptr!(mut RI_BASE + 0x14);
const RI_ERROR: *mut u32 = io_ptr!(mut RI_BASE + 0x18);
const RI_BANK_STATUS: *mut u32 = io_ptr!(mut RI_BASE + 0x1C);
const RI_BB_MODE: *mut u32 = io_ptr!(mut RI_BASE + 0x20);

pub struct Ri;

impl Ri {
    const fn new() -> Self {
        Self {}
    }

    pub fn mode(&self) -> u32 {
        unsafe { RI_MODE.read_volatile() }
    }

    pub fn config(&self) -> u32 {
        unsafe { RI_CONFIG.read_volatile() }
    }

    pub fn current_load(&self) -> u32 {
        unsafe { RI_CURRENT_LOAD.read_volatile() }
    }

    pub fn select(&self) -> u32 {
        unsafe { RI_SELECT.read_volatile() }
    }

    pub fn refresh(&self) -> u32 {
        unsafe { RI_REFRESH.read_volatile() }
    }

    pub fn latency(&self) -> u32 {
        unsafe { RI_LATENCY.read_volatile() }
    }

    pub fn error(&self) -> u32 {
        unsafe { RI_ERROR.read_volatile() }
    }

    pub fn bank_status(&self) -> u32 {
        unsafe { RI_BANK_STATUS.read_volatile() }
    }

    pub fn bb_mode(&self) -> u32 {
        unsafe { RI_BB_MODE.read_volatile() }
    }

    pub fn set_mode(&mut self, val: u32) {
        unsafe { RI_MODE.write_volatile(val) }
    }

    pub fn set_config(&mut self, val: u32) {
        unsafe { RI_CONFIG.write_volatile(val) }
    }

    pub fn set_current_load(&mut self, val: u32) {
        unsafe { RI_CURRENT_LOAD.write_volatile(val) }
    }

    pub fn set_select(&mut self, val: u32) {
        unsafe { RI_SELECT.write_volatile(val) }
    }

    pub fn set_refresh(&mut self, val: u32) {
        unsafe { RI_REFRESH.write_volatile(val) }
    }

    pub fn set_latency(&mut self, val: u32) {
        unsafe { RI_LATENCY.write_volatile(val) }
    }

    pub fn set_error(&mut self, val: u32) {
        unsafe { RI_ERROR.write_volatile(val) }
    }

    pub fn set_bank_status(&mut self, val: u32) {
        unsafe { RI_BANK_STATUS.write_volatile(val) }
    }

    pub fn set_bb_mode(&mut self, val: u32) {
        unsafe { RI_BB_MODE.write_volatile(val) }
    }

    pub fn unknown(&self, offset: u32) -> u32 {
        unsafe { io_ptr!(mut RI_BASE + offset).read_volatile() }
    }

    pub fn set_unknown(&mut self, offset: u32, val: u32) {
        unsafe { io_ptr!(mut RI_BASE + offset).write_volatile(val) }
    }
}

static mut RI: Ri = Ri::new();

pub fn ri() -> &'static mut Ri {
    unsafe { &mut RI }
}
