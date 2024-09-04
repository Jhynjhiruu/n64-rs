pub struct Burst;

impl Burst {
    pub const fn burst_start(val: u16) -> u32 {
        (val as u32 & 0x03FF) << 20
    }

    pub const fn vsync_width(val: u8) -> u32 {
        (val as u32 & 0x0F) << 16
    }

    pub const fn burst_width(val: u8) -> u32 {
        (val as u32) << 8
    }

    pub const fn hsync_width(val: u8) -> u32 {
        (val as u32) << 0
    }
}