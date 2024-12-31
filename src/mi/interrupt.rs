pub struct Interrupt;

impl Interrupt {
    pub const fn dp(val: bool) -> u32 {
        (val as u32) << 5
    }

    pub const fn pi(val: bool) -> u32 {
        (val as u32) << 4
    }

    pub const fn vi(val: bool) -> u32 {
        (val as u32) << 3
    }

    pub const fn ai(val: bool) -> u32 {
        (val as u32) << 2
    }

    pub const fn si(val: bool) -> u32 {
        (val as u32) << 1
    }

    pub const fn sp(val: bool) -> u32 {
        (val as u32) << 0
    }
}
