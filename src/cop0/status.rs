#![allow(clippy::unusual_byte_groupings)]

pub struct Status;

#[repr(u32)]
pub enum ExecutionMode {
    User = 0b10 << 3,
    Supervisor = 0b01 << 3,
    Kernel = 0b00 << 3,
}

impl Status {
    pub const SW0: u32 = (1 << 0) << 8;
    pub const SW1: u32 = (1 << 1) << 8;
    pub const HW0: u32 = (1 << 2) << 8;
    pub const HW1: u32 = (1 << 3) << 8;
    pub const HW2: u32 = (1 << 4) << 8;
    pub const HW3: u32 = (1 << 5) << 8;
    pub const HW4: u32 = (1 << 6) << 8;
    pub const TMR: u32 = (1 << 7) << 8;
    
    pub const DS: u32 = 0b0000000_111111111_0000000000000000;
    pub const IM: u32 = 0b0000000000000000_11111111_00000000;

    pub const fn coprocessor_usable(num: u8) -> u32 {
        (1 << (num as u32 & 0x03)) << 28
    }

    pub const fn reduced_power(val: bool) -> u32 {
        (val as u32) << 27
    }

    pub const fn more_float_registers(val: bool) -> u32 {
        (val as u32) << 26
    }

    pub const fn reverse_endian(val: bool) -> u32 {
        (val as u32) << 25
    }

    pub const fn diagnostic_status(val: u32) -> u32 {
        val & Self::DS
    }

    pub const fn interrupt_mask(val: u32) -> u32 {
        val & Self::IM
    }

    pub const fn kernel_addressing_64_bit(val: bool) -> u32 {
        (val as u32) << 7
    }

    pub const fn supervisor_addressing_64_bit(val: bool) -> u32 {
        (val as u32) << 6
    }

    pub const fn user_addressing_64_bit(val: bool) -> u32 {
        (val as u32) << 5
    }

    pub const fn mode(val: ExecutionMode) -> u32 {
        val as u32
    }

    pub const fn error(val: bool) -> u32 {
        (val as u32) << 2
    }

    pub const fn exception(val: bool) -> u32 {
        (val as u32) << 1
    }

    pub const fn interrupt_enable(val: bool) -> u32 {
        (val as u32) << 0
    }
}
