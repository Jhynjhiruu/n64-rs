pub struct HSync;

impl HSync {
    pub const fn leap(val: u8) -> u32 {
        (val as u32 & 0x1F) << 16
    }

    pub const fn h_sync(val: u16) -> u32 {
        (val as u32 & 0x0FFF) << 0
    }
}
