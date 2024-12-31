use core::arch::asm;

mod cause;
mod diag_status;
mod status;

pub use cause::*;
pub use diag_status::*;
pub use status::*;

use crate::boot::μs_to_ticks;
use crate::util::k0_to_phys_u32;

pub struct Cop0 {
    int_level: u32,
}

macro_rules! cop0_read {
    ($n:expr) => {
        unsafe {
            let rv;
            asm!(
                ".set noat",
                "mfc0 {reg}, ${num}",
                ".set at",
                reg = out(reg) rv,
                num = const $n
            );
            rv
        }
    };
}

macro_rules! cop0_write {
    ($v:expr, $n:expr) => {
        unsafe {
            asm!(
                ".set noat",
                "mtc0 {reg}, ${num}",
                ".set at",
                reg = in(reg) $v,
                num = const $n
            )
        }
    };
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum WatchType {
    Read = 0b10,
    Write = 0b01,
    Access = 0b11,
}

impl Cop0 {
    const fn new() -> Self {
        Self { int_level: 0 }
    }

    pub fn disable_interrupts(&mut self) {
        if self.int_level == 0 {
            self.set_status(self.status() & !Status::interrupt_enable(true));
        }
        self.int_level += 1;
    }

    pub fn enable_interrupts(&mut self) {
        if self.int_level > 0 {
            self.int_level -= 1;
        }

        if self.int_level == 0 {
            self.set_status(self.status() | Status::interrupt_enable(true));
        }
    }

    pub fn set_watch(&mut self, addr: u32, watch_type: WatchType) {
        let val = k0_to_phys_u32(addr) & !7;
        self.set_watch_lo(val | (watch_type as u32));
        self.set_watch_hi(0);
    }

    pub fn clear_watch(&mut self) {
        self.set_watch_lo(0);
        self.set_watch_hi(0);
    }

    pub fn count(&self) -> u32 {
        cop0_read!(9)
    }

    pub fn compare(&self) -> u32 {
        cop0_read!(11)
    }

    pub fn status(&self) -> u32 {
        cop0_read!(12)
    }

    pub fn cause(&self) -> u32 {
        cop0_read!(13)
    }

    pub fn epc(&self) -> u32 {
        cop0_read!(14)
    }

    pub fn watch_lo(&self) -> u32 {
        cop0_read!(18)
    }

    pub fn watch_hi(&self) -> u32 {
        cop0_read!(19)
    }

    pub fn error_epc(&self) -> u32 {
        cop0_read!(30)
    }

    pub fn set_count(&mut self, val: u32) {
        cop0_write!(val, 9)
    }

    pub fn set_compare(&mut self, val: u32) {
        cop0_write!(val, 11)
    }

    pub fn set_status(&mut self, val: u32) {
        cop0_write!(val, 12)
    }

    pub fn set_cause(&mut self, val: u32) {
        cop0_write!(val, 13)
    }

    pub fn set_epc(&mut self, val: u32) {
        cop0_write!(val, 14)
    }

    pub fn set_watch_lo(&mut self, val: u32) {
        cop0_write!(val, 18)
    }

    pub fn set_watch_hi(&mut self, val: u32) {
        cop0_write!(val, 19)
    }

    pub fn set_error_epc(&mut self, val: u32) {
        cop0_write!(val, 30)
    }

    pub fn delay(&self, microsecs: u32) {
        let start = self.count();
        let ticks = μs_to_ticks(microsecs);

        while self.count().wrapping_sub(start) < ticks {}
    }
}

static mut COP0: Cop0 = Cop0::new();

pub fn cop0() -> &'static mut Cop0 {
    unsafe { &mut COP0 }
}
