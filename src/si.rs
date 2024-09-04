use core::{hint::black_box, mem::MaybeUninit};

use crate::{
    data_cache_invalidate, data_cache_writeback, io_ptr,
    util::{k0_to_phys, k0_to_phys_mut},
};

const SI_BASE: u32 = 0x0480_0000;

const SI_DRAM_ADDR: *mut u32 = io_ptr!(mut SI_BASE + 0x00);
const SI_PIF_AD_RD64B: *mut u32 = io_ptr!(mut SI_BASE + 0x04);
const SI_PIF_AD_WR4B: *mut u32 = io_ptr!(mut SI_BASE + 0x08);

const SI_PIF_AD_WR64B: *mut u32 = io_ptr!(mut SI_BASE + 0x10);
const SI_PIF_AD_RD4B: *mut u32 = io_ptr!(mut SI_BASE + 0x14);
const SI_STATUS: *mut u32 = io_ptr!(mut SI_BASE + 0x18);

pub struct Si;

impl Si {
    const PIF_RAM_START: u32 = 0x1FC0_07C0;

    const fn new() -> Self {
        Self {}
    }

    pub fn wait(&self) {
        while self.status() & 3 != 0 {}
    }

    pub fn write(&mut self, data: &[u8; 64]) {
        data_cache_writeback(data);

        self.wait();

        self.set_dram_addr(k0_to_phys(data.as_ptr()).addr() as _);
        self.set_pif_ad_wr64b(Self::PIF_RAM_START);

        self.wait();
    }

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

    pub fn dram_addr(&self) -> u32 {
        unsafe { SI_DRAM_ADDR.read_volatile() }
    }

    pub fn pif_ad_rd64b(&self) -> u32 {
        unsafe { SI_PIF_AD_RD64B.read_volatile() }
    }

    pub fn pif_ad_wr4b(&self) -> u32 {
        unsafe { SI_PIF_AD_WR4B.read_volatile() }
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

    pub fn set_dram_addr(&mut self, val: u32) {
        unsafe { SI_DRAM_ADDR.write_volatile(val) }
    }

    pub fn set_pif_ad_rd64b(&mut self, val: u32) {
        unsafe { SI_PIF_AD_RD64B.write_volatile(val) }
    }

    pub fn set_pif_ad_wr4b(&mut self, val: u32) {
        unsafe { SI_PIF_AD_WR4B.write_volatile(val) }
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
}

static mut SI: Si = Si::new();

pub fn si() -> &'static mut Si {
    unsafe { &mut SI }
}
