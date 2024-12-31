use core::{arch::asm, hint::black_box, mem::MaybeUninit, num::Wrapping};

use crate::{
    data_cache_invalidate, data_cache_writeback, io_ptr,
    util::{k0_to_phys, k0_to_phys_mut},
};

const SI_BASE: u32 = 0x0480_0000;

const SI_DRAM_ADDR: *mut u32 = io_ptr!(mut SI_BASE + 0x00);
const SI_PIF_AD_RD64B: *mut u32 = io_ptr!(mut SI_BASE + 0x04);
const SI_PIF_AD_WR4B: *mut u32 = io_ptr!(mut SI_BASE + 0x08);
const SI_CTRL: *mut u32 = io_ptr!(mut SI_BASE + 0x0C);
const SI_PIF_AD_WR64B: *mut u32 = io_ptr!(mut SI_BASE + 0x10);
const SI_PIF_AD_RD4B: *mut u32 = io_ptr!(mut SI_BASE + 0x14);
const SI_STATUS: *mut u32 = io_ptr!(mut SI_BASE + 0x18);
const SI_CONFIG: *mut u32 = io_ptr!(mut SI_BASE + 0x0C);

/*#[cfg(feature = "sk")]
#[link_section = ".dram"]
pub static mut DRAM_BUF: [u8; 64] = unsafe { MaybeUninit::zeroed().assume_init() };*/

pub struct Si {
    pub(crate) tx_index: Wrapping<u8>,
    pub(crate) rx_index: Wrapping<u8>,
}

impl Si {
    const PIF_RAM_START: u32 = 0x1FC0_07C0;

    const fn new() -> Self {
        Self {
            tx_index: Wrapping(0),
            rx_index: Wrapping(0),
        }
    }

    pub fn wait(&self) {
        while self.status() & 3 != 0 {}
    }

    pub fn init_hw(&mut self) {
        self.set_ctrl(0);
        self.set_config((self.config() & !0x7F000000) | (47 << 24) | (1 << 22));
        //self.set_config((0 << 31) | (47 << 24) | (0 << 23) | (1 << 22) | (1 << 16))
    }

    #[cfg(not(feature = "sk"))]
    pub fn write(&mut self, data: &[u8; 64]) {
        data_cache_writeback(data);

        self.wait();

        self.set_dram_addr(k0_to_phys(data.as_ptr()).addr() as _);
        self.set_pif_ad_wr64b(Self::PIF_RAM_START);

        self.wait();
    }

    /*#[cfg(feature = "sk")]
    #[inline(never)]
    pub fn write(&mut self) {
        unsafe {
            asm!(
                ".set noreorder                 ",
                ".set noat                      ",
                "                               ",
                "   la {t0}, {buf}              ",
                "   cache 0x19, 0x00({t0})      ",
                "   cache 0x19, 0x10({t0})      ",
                "   cache 0x19, 0x20({t0})      ",
                "   cache 0x19, 0x30({t0})      ",
                "                               ",
                "1:                             ",
                "   lw {t0}, 0({status})        ",
                "   and {t0}, 3                 ",
                "   bnez {t0}, 1b               ",
                "    nop                        ",
                "                               ",
                "   li {t0}, 0x1FFFFFFF         ",
                "   la {t1}, {buf}              ",
                "   and {t0}, {t0}, {t1}        ",
                "   sw {t0}, 0({dram_addr})     ",
                "                               ",
                "   li {t0}, {pif_ram_start}    ",
                "   sw {t0}, 0({pif_ad_wr64b})  ",
                "                               ",
                "1:                             ",
                "   lw {t0}, 0({status})        ",
                "   and {t0}, 3                 ",
                "   bnez {t0}, 1b               ",
                "    nop                        ",
                "                               ",
                ".set at                        ",
                ".set reorder                   ",
                t0 = out(reg) _,
                buf = sym DRAM_BUF,
                status = in(reg) SI_STATUS,
                t1 = out(reg) _,
                dram_addr = in(reg) SI_DRAM_ADDR,
                pif_ram_start = const Self::PIF_RAM_START,
                pif_ad_wr64b = in(reg) SI_PIF_AD_WR64B,
                options(nostack)
            )
        }
        /*data_cache_writeback(&unsafe { DRAM_BUF });

        self.wait();

        self.set_dram_addr(0);
        self.set_pif_ad_wr64b(Self::PIF_RAM_START);

        self.wait();*/
    }*/

    #[cfg(not(feature = "sk"))]
    pub fn read(&mut self) -> [u8; 64] {
        let mut buf = [0xA5; 64];

        //data_cache_writeback(&buf);

        self.wait();

        self.set_dram_addr(k0_to_phys_mut(buf.as_mut_ptr()).addr() as _);
        self.set_pif_ad_rd64b(Self::PIF_RAM_START);

        self.wait();

        data_cache_invalidate(&buf);

        buf
    }

    /*#[cfg(feature = "sk")]
    pub fn read(&mut self) {
        unsafe {
            asm!(
                ".set noreorder                 ",
                ".set noat                      ",
                "                               ",
                "1:                             ",
                "   lw {t0}, 0({status})        ",
                "   and {t0}, 3                 ",
                "   bnez {t0}, 1b               ",
                "    nop                        ",
                "                               ",
                "   li {t0}, 0x1FFFFFFF         ",
                "   la {t1}, {buf}              ",
                "   and {t0}, {t0}, {t1}        ",
                "   sw {t0}, 0({dram_addr})     ",
                "                               ",
                "   li {t0}, {pif_ram_start}    ",
                "   sw {t0}, 0({pif_ad_rd64b})  ",
                "                               ",
                "1:                             ",
                "   lw {t0}, 0({status})        ",
                "   and {t0}, 3                 ",
                "   bnez {t0}, 1b               ",
                "    nop                        ",
                "                               ",
                "   la {t0}, {buf}              ",
                "   cache 0x11, 0x00({t0})      ",
                "   cache 0x11, 0x10({t0})      ",
                "   cache 0x11, 0x20({t0})      ",
                "   cache 0x11, 0x30({t0})      ",
                "                               ",
                ".set at                        ",
                ".set reorder                   ",
                status = in(reg) SI_STATUS,
                t0 = out(reg) _,
                t1 = out(reg) _,
                buf = sym DRAM_BUF,
                dram_addr = in(reg) SI_DRAM_ADDR,
                pif_ram_start = const Self::PIF_RAM_START,
                pif_ad_rd64b = in(reg) SI_PIF_AD_RD64B,
                options(nostack)
            )
        }
        //data_cache_writeback(&buf);

        /*self.wait();

        self.set_dram_addr(0);
        self.set_pif_ad_rd64b(Self::PIF_RAM_START);

        self.wait();

        data_cache_invalidate(&unsafe { DRAM_BUF });*/
    }*/

    pub fn dram_addr(&self) -> u32 {
        unsafe { SI_DRAM_ADDR.read_volatile() }
    }

    pub fn pif_ad_rd64b(&self) -> u32 {
        unsafe { SI_PIF_AD_RD64B.read_volatile() }
    }

    pub fn pif_ad_wr4b(&self) -> u32 {
        unsafe { SI_PIF_AD_WR4B.read_volatile() }
    }

    pub fn ctrl(&self) -> u32 {
        unsafe { SI_CTRL.read_volatile() }
    }

    pub fn pif_ad_wr64b(&self) -> u32 {
        unsafe { SI_PIF_AD_WR64B.read_volatile() }
    }

    pub fn pif_ad_rd4b(&self) -> u32 {
        unsafe { SI_PIF_AD_RD4B.read_volatile() }
    }

    pub fn status(&self) -> u32 {
        unsafe { SI_STATUS.read_volatile() }
    }

    pub fn config(&self) -> u32 {
        unsafe { SI_CONFIG.read_volatile() }
    }

    pub fn set_dram_addr(&mut self, val: u32) {
        unsafe { SI_DRAM_ADDR.write_volatile(val) }
    }

    pub fn set_pif_ad_rd64b(&mut self, val: u32) {
        unsafe { SI_PIF_AD_RD64B.write_volatile(val) }
    }

    pub fn set_pif_ad_wr4b(&mut self, val: u32) {
        unsafe { SI_PIF_AD_WR4B.write_volatile(val) }
    }

    pub fn set_ctrl(&mut self, val: u32) {
        unsafe { SI_CTRL.write_volatile(val) }
    }

    pub fn set_pif_ad_wr64b(&mut self, val: u32) {
        unsafe { SI_PIF_AD_WR64B.write_volatile(val) }
    }

    pub fn set_pif_ad_rd4b(&mut self, val: u32) {
        unsafe { SI_PIF_AD_RD4B.write_volatile(val) }
    }

    pub fn set_status(&mut self, val: u32) {
        unsafe { SI_STATUS.write_volatile(val) }
    }

    pub fn set_config(&mut self, val: u32) {
        unsafe { SI_CONFIG.write_volatile(val) }
    }
}

static mut SI: Si = Si::new();

pub fn si() -> &'static mut Si {
    unsafe { &mut SI }
}
