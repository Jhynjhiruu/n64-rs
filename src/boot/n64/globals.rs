use volcell::VolatileCell;

use crate::{mi::mi, pi::pi};

extern "C" {
    pub static mut osTvType: u32;
    pub static mut osRomType: u32;
    pub static mut osRomBase: u32;
    pub static mut osResetType: u32;
    pub static mut osCicId: u32;
    pub static mut osVersion: u32;
    pub static mut osMemSize: u32;
    pub static mut osAppNMIBuffer: [u8; 64];

    pub static mut __osBbEepromAddress: u32;
    pub static mut __osBbEepromSize: u32;
    pub static mut __osBbFlashAddress: u32;
    pub static mut __osBbFlashSize: u32;
    pub static mut __osBbSramAddress: u32;
    pub static mut __osBbSramSize: u32;
    pub static mut __osBbPakAddress: [u32; 4];
    pub static mut __osBbPakSize: u32;
    pub static mut __osBbIsBb: u32;
    pub static mut __osBbHackFlags: u32;
    pub static mut __osBbStashMagic: u32;
    pub static mut __osBbPakBindings: [u32; 4];
    pub static mut __osBbPakStateName: [u8; 16];
    pub static mut __osBbStateDirty: u32;
    pub static mut __osBbAuxDataLimit: u32;
}

pub(super) fn setup_globals() {
    let mi = mi();
    let pi = pi();
    unsafe {
        (&raw mut __osBbIsBb).write_volatile(if mi.version() & 0xF0 == 0xB0 {
            if pi.bb_gpio() & 0xC0000000 != 0 {
                2
            } else {
                1
            }
        } else {
            0
        });
    }
}
