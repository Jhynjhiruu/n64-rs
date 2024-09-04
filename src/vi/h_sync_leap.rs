pub struct HSyncLeap;

impl HSyncLeap {
    pub const fn leap_a(val: u16) -> u32 {
        (val as u32 & 0x0FFF) << 16
    }

    pub const fn leap_b(val: u16) -> u32 {
        (val as u32 & 0x0FFF) << 0
    }
}
