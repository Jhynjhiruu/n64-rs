use core::{
    array::from_fn,
    mem::{size_of, size_of_val},
};

use crate::{
    boot::globals::osRomBase,
    data_cache_invalidate, data_cache_writeback, io_ptr,
    util::{k0_to_phys, k0_to_phys_mut, k0_to_phys_u32, k0_to_phys_usize},
};

const PI_BASE: u32 = 0x0460_0000;

const PI_DRAM_ADDR: *mut u32 = io_ptr!(mut PI_BASE + 0x00);
const PI_CART_ADDR: *mut u32 = io_ptr!(mut PI_BASE + 0x04);
const PI_RD_LEN: *mut u32 = io_ptr!(mut PI_BASE + 0x08);
const PI_WR_LEN: *mut u32 = io_ptr!(mut PI_BASE + 0x0C);
const PI_STATUS: *mut u32 = io_ptr!(mut PI_BASE + 0x10);

const PI_BB_ATB_UPPER: *mut u32 = io_ptr!(mut PI_BASE + 0x40);
const PI_BB_NAND_CTRL: *mut u32 = io_ptr!(mut PI_BASE + 0x48);
const PI_BB_NAND_CFG: *mut u32 = io_ptr!(mut PI_BASE + 0x4C);

const PI_BB_RD_LEN: *mut u32 = io_ptr!(mut PI_BASE + 0x58);
const PI_BB_WR_LEN: *mut u32 = io_ptr!(mut PI_BASE + 0x5C);
const PI_BB_GPIO: *mut u32 = io_ptr!(mut PI_BASE + 0x60);

const PI_BB_NAND_ADDR: *mut u32 = io_ptr!(mut PI_BASE + 0x70);

const PI_BB_ATB_LOWER: *mut [u32] = io_ptr!(mut PI_BASE + 0x500; 192);

#[repr(u32)]
pub enum LedValue {
    On = 0,
    Off = 1,
}

pub struct Pi;

impl Pi {
    const fn new() -> Self {
        Self {}
    }

    pub fn wait(&self) {
        while self.status() & 3 != 0 {}
    }

    pub fn write<T>(&mut self, data: &[T], addr: u32) {
        let len = size_of_val(data);

        assert!(
            data.as_ptr().addr() % 8 == 0,
            "RAM address must be 8-byte aligned, otherwise behaviour is not well-defined"
        );
        assert!(
            addr % 2 == 0,
            "PI address must be 2-byte aligned, otherwise behaviour is not well-defined"
        );
        assert!(
            len % 2 == 0,
            "Length must be a multiple of 2, otherwise behaviour is not well-defined"
        );

        data_cache_writeback(data);

        self.wait();

        self.set_dram_addr(k0_to_phys(data.as_ptr()).addr() as _);
        self.set_cart_addr(k0_to_phys_u32(addr | unsafe { osRomBase.read() }));
        self.set_rd_len((len - 1) as _);

        self.wait();
    }

    pub fn read<T: Default, const N: usize>(&mut self, addr: u32) -> [T; N] {
        let mut buf = from_fn(|_| Default::default());

        self.read_into(&mut buf, addr);

        buf
    }

    pub fn read_into<T>(&mut self, data: &mut [T], addr: u32) {
        let len = size_of_val(data);

        assert!(
            data.as_ptr().addr() % 8 == 0,
            "RAM address must be 8-byte aligned, otherwise behaviour is not well-defined"
        );
        assert!(
            addr % 2 == 0,
            "PI address must be 2-byte aligned, otherwise behaviour is not well-defined"
        );
        assert!(
            len % 2 == 0,
            "Length must be a multiple of 2, otherwise behaviour is not well-defined"
        );

        self.wait();

        self.set_dram_addr(k0_to_phys_mut(data.as_mut_ptr()).addr() as _);
        self.set_cart_addr(k0_to_phys_u32(addr | unsafe { osRomBase.read() }));
        self.set_wr_len((len - 1) as _);

        self.wait();

        data_cache_invalidate(data)
    }

    pub fn dram_addr(&self) -> u32 {
        unsafe { PI_DRAM_ADDR.read_volatile() }
    }

    pub fn cart_addr(&self) -> u32 {
        unsafe { PI_CART_ADDR.read_volatile() }
    }

    pub fn rd_len(&self) -> u32 {
        unsafe { PI_RD_LEN.read_volatile() }
    }

    pub fn wr_len(&self) -> u32 {
        unsafe { PI_WR_LEN.read_volatile() }
    }

    pub fn status(&self) -> u32 {
        unsafe { PI_STATUS.read_volatile() }
    }

    pub fn bb_atb_upper(&self) -> u32 {
        unsafe { PI_BB_ATB_UPPER.read_volatile() }
    }

    pub fn bb_nand_ctrl(&self) -> u32 {
        unsafe { PI_BB_NAND_CTRL.read_volatile() }
    }

    pub fn bb_nand_cfg(&self) -> u32 {
        unsafe { PI_BB_NAND_CFG.read_volatile() }
    }

    pub fn bb_rd_len(&self) -> u32 {
        unsafe { PI_BB_RD_LEN.read_volatile() }
    }

    pub fn bb_wr_len(&self) -> u32 {
        unsafe { PI_BB_WR_LEN.read_volatile() }
    }

    pub fn bb_gpio(&self) -> u32 {
        unsafe { PI_BB_GPIO.read_volatile() }
    }

    pub fn bb_nand_addr(&self) -> u32 {
        unsafe { PI_BB_NAND_ADDR.read_volatile() }
    }

    pub fn set_dram_addr(&mut self, val: u32) {
        unsafe { PI_DRAM_ADDR.write_volatile(val) }
    }

    pub fn set_cart_addr(&mut self, val: u32) {
        unsafe { PI_CART_ADDR.write_volatile(val) }
    }

    pub fn set_rd_len(&mut self, val: u32) {
        unsafe { PI_RD_LEN.write_volatile(val) }
    }

    pub fn set_wr_len(&mut self, val: u32) {
        unsafe { PI_WR_LEN.write_volatile(val) }
    }

    pub fn set_status(&mut self, val: u32) {
        unsafe { PI_STATUS.write_volatile(val) }
    }

    pub fn set_bb_atb_upper(&mut self, val: u32) {
        unsafe { PI_BB_ATB_UPPER.write_volatile(val) }
    }

    pub fn set_bb_nand_ctrl(&mut self, val: u32) {
        unsafe { PI_BB_NAND_CTRL.write_volatile(val) }
    }

    pub fn set_bb_nand_cfg(&mut self, val: u32) {
        unsafe { PI_BB_NAND_CFG.write_volatile(val) }
    }

    pub fn set_bb_rd_len(&mut self, val: u32) {
        unsafe { PI_BB_RD_LEN.write_volatile(val) }
    }

    pub fn set_bb_wr_len(&mut self, val: u32) {
        unsafe { PI_BB_WR_LEN.write_volatile(val) }
    }

    pub fn set_bb_gpio(&mut self, val: u32) {
        unsafe { PI_BB_GPIO.write_volatile(val) }
    }

    pub fn set_bb_nand_addr(&mut self, val: u32) {
        unsafe { PI_BB_NAND_ADDR.write_volatile(val) }
    }

    pub fn set_led(&mut self, val: LedValue) {
        let prev = self.bb_gpio() & !(1 << 1);
        let new = prev | (1 << 5) | ((val as u32) << 1);
        self.set_bb_gpio(new);
    }
}

static mut PI: Pi = Pi::new();

pub fn pi() -> &'static mut Pi {
    unsafe { &mut PI }
}
