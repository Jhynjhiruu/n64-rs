pub struct Video;

impl Video {
    pub const fn start(val: u16) -> u32 {
        (val as u32 & 0x03FF) << 16
    }

    pub const fn end(val: u16) -> u32 {
        (val as u32 & 0x03FF) << 0
    }
}
