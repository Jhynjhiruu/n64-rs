#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]
#![feature(ptr_metadata)]
#![feature(ptr_as_uninit)]
#![feature(const_trait_impl)]
#![feature(naked_functions)]
#![feature(generic_const_exprs)]

use core::arch::asm;
use core::ops::Range;

pub mod aes;
pub mod boot;
pub mod card;
pub mod cop0;
pub mod joybus;
pub mod mi;
#[cfg(feature = "alloc")]
mod n64_alloc;
pub mod pi;
pub mod recrypt;
pub mod ri;
pub mod si;
pub mod skapi;
pub mod text;
pub mod types;
pub mod usb;
pub mod util;
pub mod v2;
pub mod vi;

#[macro_export]
macro_rules! io_ptr {
    (mut $e:expr) => {
        core::ptr::from_raw_parts_mut::<u32>($crate::util::phys_to_k1_u32($e) as *mut (), ())
    };
    (mut $e:expr; $n:expr) => {
        core::ptr::from_raw_parts_mut::<[u32]>($crate::util::phys_to_k1_u32($e) as *mut (), $n)
    };
}

macro_rules! cache {
    (data, $n:expr, $e:expr) => {
        unsafe {
            asm!(
                ".set noat",
                "cache {num}, 0({reg})",
                ".set at",
                num = const ($n << 2) | 1,
                reg = in(reg) $e
            )
        }
    };
    (instruction, $n:expr, $e:expr) => {
        unsafe {
            asm!(
                ".set noat",
                "cache {num}, 0({reg})",
                ".set at",
                num = const $n << 2,
                reg = in(reg) $e
            )
        }
    };
}

pub fn data_cache_writeback<T>(data: &[T]) {
    let Range { start, end } = data.as_ptr_range();

    for i in (start.addr()..end.addr()).step_by(0x10) {
        cache!(data, 6, i);
    }
}

pub fn data_cache_writeback_raw(start: usize, end: usize) {
    for i in (start..end).step_by(0x10) {
        cache!(data, 6, i);
    }
}

pub fn data_cache_invalidate<T>(data: &[T]) {
    let Range { start, end } = data.as_ptr_range();

    for i in (start.addr()..end.addr()).step_by(0x10) {
        cache!(data, 4, i);
    }
}

pub fn instruction_cache_invalidate<T>(data: &[T]) {
    let Range { start, end } = data.as_ptr_range();

    for i in (start.addr()..end.addr()).step_by(0x20) {
        cache!(instruction, 4, i);
    }
}
