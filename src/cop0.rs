use core::arch::asm;

pub struct Cop0;

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

impl Cop0 {
    const fn new() -> Self {
        Self {}
    }

    pub fn status(&self) -> u32 {
        cop0_read!(12)
    }

    pub fn cause(&self) -> u32 {
        cop0_read!(13)
    }

    pub fn set_status(&mut self, val: u32) {
        cop0_write!(val, 12)
    }

    pub fn set_cause(&mut self, val: u32) {
        cop0_write!(val, 13)
    }
}

static mut COP0: Cop0 = Cop0::new();

pub fn cop0() -> &'static mut Cop0 {
    unsafe { &mut COP0 }
}
