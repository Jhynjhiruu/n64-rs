#![allow(clippy::unusual_byte_groupings)]

use core::{
    arch::asm,
    ffi::c_void,
    hint::unreachable_unchecked,
    ptr::{addr_of, from_raw_parts, from_raw_parts_mut},
};

use crate::{
    cop0::cop0, data_cache_writeback, instruction_cache_invalidate, mi::mi, pi::pi, vi::vi,
};

use super::globals::is_bbplayer;

#[link_section = ".text"]
extern "C" fn int_handler() {
    let vi = vi();
    vi.blank();
}

#[link_section = ".int_handler"]
#[naked]
#[no_mangle]
unsafe extern "C" fn int_handler_entry() -> ! {
    asm!(
        ".set noreorder ",
        "  jal {handler}",
        "   nop         ",
        "  eret         ",
        "   nop         ",
        ".set reorder   ",
        handler = sym int_handler,
        options(noreturn)
    )
}

extern "C" {
    static int_handler_ROM_START: c_void;
    static int_handler_ROM_END: c_void;
}

pub unsafe fn setup_ints() {
    let start = addr_of!(int_handler_ROM_START).addr();
    let end = addr_of!(int_handler_ROM_END).addr();
    let len = end - start;

    let dst = from_raw_parts_mut::<[u8]>(0x80000180u32 as _, len)
        .as_mut()
        .expect("should never be null");

    let pi = pi();
    pi.read_into(dst, start as _);

    instruction_cache_invalidate(dst);

    let mi = mi();
    // disable all interrupts except button
    if is_bbplayer() {
        mi.set_bb_mask(0b00_00_01_10__01_01_01_01__01_01_01_01__01_01_01_01);
    } else {
        mi.set_mask(0b00_00_00_00__00_00_00_00__00_00_01_01__01_01_01_01)
    }

    let cop0 = cop0();
    // clear pending interrupt causes
    cop0.set_cause(cop0.cause() & 0b1_0_1_000000000000_00000000_0_11111_00);
    // enable interrupts and mask pre-NMI
    cop0.set_status(0b0011_0_0_0_000000000_00010000_0_0_0_00_0_0_1);
    //cop0.set_status(cop0.status() | 0x00000001);
}
