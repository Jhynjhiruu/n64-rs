pub struct Ctrl;

#[repr(u32)]
pub enum AAMode {
    None = 0b11 << 8,
    Resampling = 0b10 << 8,
    Both = 0b01 << 8,
    BothFetchAlways = 0b00 << 8,
}

#[repr(u32)]
pub enum PixelSize {
    Rgba8 = 0b11 << 0,
    Rgba5553 = 0b10 << 0,
    Reserved = 0b01 << 0,
    Blank = 0b00 << 0,
}

impl Ctrl {
    pub const fn dedither_enable(val: bool) -> u32 {
        (val as u32) << 16
    }

    pub const fn pixel_advance(val: u8) -> u32 {
        (val as u32 & 0x0F) << 12
    }

    pub const fn kill_we(val: bool) -> u32 {
        (val as u32) << 11
    }

    pub const fn aa_mode(val: AAMode) -> u32 {
        val as u32
    }

    pub const fn test_mode(val: bool) -> u32 {
        (val as u32) << 7
    }

    pub const fn serrate(val: bool) -> u32 {
        (val as u32) << 6
    }

    pub const fn vbus_clock_enable(val: bool) -> u32 {
        (val as u32) << 5
    }

    pub const fn divot_enable(val: bool) -> u32 {
        (val as u32) << 4
    }

    pub const fn gamma_enable(val: bool) -> u32 {
        (val as u32) << 3
    }

    pub const fn gamma_dither_enable(val: bool) -> u32 {
        (val as u32) << 2
    }

    pub const fn pixel_size(val: PixelSize) -> u32 {
        val as u32
    }
}
