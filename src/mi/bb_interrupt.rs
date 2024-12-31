pub struct BBInterrupt;

impl BBInterrupt {
    pub const fn md_state(val: bool) -> u32 {
        (val as u32) << 25
    }

    pub const fn btn_state(val: bool) -> u32 {
        (val as u32) << 24
    }

    pub const fn md(val: bool) -> u32 {
        (val as u32) << 13
    }

    pub const fn btn(val: bool) -> u32 {
        (val as u32) << 12
    }

    pub const fn usb1(val: bool) -> u32 {
        (val as u32) << 11
    }

    pub const fn usb0(val: bool) -> u32 {
        (val as u32) << 10
    }

    pub const fn pi_err(val: bool) -> u32 {
        (val as u32) << 9
    }

    pub const fn ide(val: bool) -> u32 {
        (val as u32) << 8
    }

    pub const fn aes(val: bool) -> u32 {
        (val as u32) << 7
    }

    pub const fn flash(val: bool) -> u32 {
        (val as u32) << 6
    }
}
