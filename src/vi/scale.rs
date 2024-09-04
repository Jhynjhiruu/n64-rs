use super::FixedPoint;

pub struct Scale;

impl Scale {
    pub const fn offset(val: FixedPoint) -> u32 {
        (val.to_u16() as u32 & 0x0FFF) << 16
    }

    pub const fn scale(val: FixedPoint) -> u32 {
        (val.to_u16() as u32 & 0x0FFF) << 0
    }
}
