use volcell::VolatileCell;

use crate::{mi::mi, pi::pi};

extern "C" {
    pub static mut osTvType: VolatileCell<u32>;
    pub static mut osRomType: VolatileCell<u32>;
    pub static mut osRomBase: VolatileCell<u32>;
    pub static mut osResetType: VolatileCell<u32>;
    pub static mut osCicId: VolatileCell<u32>;
    pub static mut osVersion: VolatileCell<u32>;
    pub static mut osMemSize: VolatileCell<u32>;
    pub static mut osAppNMIBuffer: [u8; 64];

    pub static mut __osBbEepromAddress: VolatileCell<u32>;
    pub static mut __osBbEepromSize: VolatileCell<u32>;
    pub static mut __osBbFlashAddress: VolatileCell<u32>;
    pub static mut __osBbFlashSize: VolatileCell<u32>;
    pub static mut __osBbSramAddress: VolatileCell<u32>;
    pub static mut __osBbSramSize: VolatileCell<u32>;
    pub static mut __osBbPakAddress: [VolatileCell<u32>; 4];
    pub static mut __osBbPakSize: VolatileCell<u32>;
    pub static mut __osBbIsBb: VolatileCell<u32>;
    pub static mut __osBbHackFlags: VolatileCell<u32>;
    pub static mut __osBbStashMagic: VolatileCell<u32>;
    pub static mut __osBbPakBindings: [VolatileCell<u32>; 4];
    pub static mut __osBbPakStateName: [u8; 16];
    pub static mut __osBbStateDirty: VolatileCell<u32>;
    pub static mut __osBbAuxDataLimit: VolatileCell<u32>;
}

pub fn is_bbplayer() -> bool {
    unsafe { __osBbIsBb.read() != 0 }
}

pub(super) fn setup_globals() {
    let mi = mi();
    let pi = pi();
    unsafe {
        __osBbIsBb.write(if mi.version() & 0xF0 == 0xB0 {
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
