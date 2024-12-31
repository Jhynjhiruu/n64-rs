#![allow(clippy::unusual_byte_groupings)]

pub struct Cause;

#[repr(u32)]
pub enum ExceptionCode {
    Interrupt = 0 << 2,
    TLBModification = 1 << 2,
    TLBMissLoad = 2 << 2,
    TLBMissStore = 3 << 2,
    AddressErrorLoad = 4 << 2,
    AddressErrorStore = 5 << 2,
    InstructionBusError = 6 << 2,
    DataBusError = 7 << 2,
    Syscall = 8 << 2,
    Breakpoint = 9 << 2,
    ReservedInstruction = 10 << 2,
    CoprocessorUnusable = 11 << 2,
    ArithmeticOverflow = 12 << 2,
    Trap = 13 << 2,

    FloatingPoint = 15 << 2,

    Watch = 23 << 2,
}

impl Cause {
    pub const SW0: u32 = (1 << 0) << 8;
    pub const SW1: u32 = (1 << 1) << 8;
    pub const HW0: u32 = (1 << 2) << 8;
    pub const HW1: u32 = (1 << 3) << 8;
    pub const HW2: u32 = (1 << 4) << 8;
    pub const HW3: u32 = (1 << 5) << 8;
    pub const HW4: u32 = (1 << 6) << 8;
    pub const TMR: u32 = (1 << 7) << 8;

    pub const CE: u32 = 0b00_11_0000000000000000000000000000;
    pub const IP: u32 = 0b0000000000000000_11111111_00000000;
    pub const EC: u32 = 0b0000000000000000000000000_11111_00;

    pub const fn branch_delay(val: bool) -> u32 {
        (val as u32) << 31
    }

    pub const fn coprocessor_number(num: u8) -> u32 {
        (1 << (num as u32 & 0x03)) << 28
    }

    pub const fn interrupt_pending(val: u32) -> u32 {
        val & Self::IP
    }

    pub const fn exception_code(val: ExceptionCode) -> u32 {
        val as u32
    }
}
