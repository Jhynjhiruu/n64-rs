use core::{
    array::from_fn,
    mem::{size_of, size_of_val},
    ops::Not,
};

#[cfg(not(feature = "sk"))]
use crate::boot::globals::osRomBase;
use crate::types::Align8;
use crate::util::{k0_to_phys, k0_to_phys_mut, k0_to_phys_u32, k0_to_phys_usize};
use crate::{data_cache_invalidate, data_cache_writeback, io_ptr};

const PI_BASE: u32 = 0x0460_0000;

const PI_DRAM_ADDR: *mut u32 = io_ptr!(mut PI_BASE + 0x00);
const PI_CART_ADDR: *mut u32 = io_ptr!(mut PI_BASE + 0x04);
const PI_RD_LEN: *mut u32 = io_ptr!(mut PI_BASE + 0x08);
const PI_WR_LEN: *mut u32 = io_ptr!(mut PI_BASE + 0x0C);
const PI_STATUS: *mut u32 = io_ptr!(mut PI_BASE + 0x10);

const PI_BB_ATB_UPPER: *mut u32 = io_ptr!(mut PI_BASE + 0x40);
const PI_BB_NAND_CTRL: *mut u32 = io_ptr!(mut PI_BASE + 0x48);
const PI_BB_NAND_CFG: *mut u32 = io_ptr!(mut PI_BASE + 0x4C);
const PI_BB_AES_CTRL: *mut u32 = io_ptr!(mut PI_BASE + 0x50);
const PI_BB_ALLOWED_IO: *mut u32 = io_ptr!(mut PI_BASE + 0x54);
const PI_BB_RD_LEN: *mut u32 = io_ptr!(mut PI_BASE + 0x58);
const PI_BB_WR_LEN: *mut u32 = io_ptr!(mut PI_BASE + 0x5C);
const PI_BB_GPIO: *mut u32 = io_ptr!(mut PI_BASE + 0x60);
const PI_BB_IDE_CONFIG: *mut u32 = io_ptr!(mut PI_BASE + 0x64);
const PI_BB_IDE_CTRL: *mut u32 = io_ptr!(mut PI_BASE + 0x68);

const PI_BB_NAND_ADDR: *mut u32 = io_ptr!(mut PI_BASE + 0x70);

const PI_BB_ATB_LOWER: *mut [u32] = io_ptr!(mut PI_BASE + 0x500; 192);

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LedValue {
    On = 0,
    Off = 1,
}

impl Not for LedValue {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}

pub struct Pi;

impl Pi {
    const fn new() -> Self {
        Self {}
    }

    pub fn wait(&self) {
        while self.status() & 3 != 0 {}
    }

    pub fn init_hw(&mut self) {
        self.set_bb_ide_config(self.bb_ide_config() & !0x80000000);
    }

    #[cfg(not(feature = "sk"))]
    #[track_caller]
    pub fn write<T>(&mut self, data: &Align8<[T]>, addr: u32) {
        let len = size_of_val(&data.0);

        assert!(
            addr % 2 == 0,
            "PI address ({addr:08X}) must be 2-byte aligned, otherwise behaviour is not well-defined"
        );
        assert!(
            len % 2 == 0,
            "Length ({len:X}) must be a multiple of 2, otherwise behaviour is not well-defined"
        );

        data_cache_writeback(&data.0);

        self.wait();

        self.set_dram_addr(k0_to_phys(data.0.as_ptr()).addr() as _);
        self.set_cart_addr(addr);
        self.set_rd_len((len - 1) as _);

        self.wait();
    }

    #[cfg(not(feature = "sk"))]
    pub fn read<T: Default, const N: usize>(&mut self, addr: u32) -> [T; N] {
        let mut buf = Align8(from_fn(|_| Default::default()));

        self.read_into(&mut buf, addr);

        buf.0
    }

    #[cfg(not(feature = "sk"))]
    #[track_caller]
    pub fn read_into<T>(&mut self, data: &mut Align8<[T]>, addr: u32) {
        let len = size_of_val(&data.0);

        data_cache_invalidate(&data.0);

        assert!(
            addr % 2 == 0,
            "PI address ({addr:08X}) must be 2-byte aligned, otherwise behaviour is not well-defined"
        );
        assert!(
            len % 2 == 0,
            "Length ({len:X}) must be a multiple of 2, otherwise behaviour is not well-defined"
        );

        self.wait();

        self.set_dram_addr(k0_to_phys_mut(data.0.as_mut_ptr()).addr() as _);
        self.set_cart_addr(addr);
        self.set_wr_len((len - 1) as _);

        self.wait();

        data_cache_invalidate(&data.0)
    }

    #[track_caller]
    pub fn bb_write<T>(&mut self, data: &Align8<[T]>, addr: u32) {
        let len = size_of_val(&data.0);

        assert!(
            addr % 2 == 0,
            "PI address ({addr:08X}) must be 2-byte aligned, otherwise behaviour is not well-defined"
        );
        assert!(
            len % 2 == 0,
            "Length ({len:X}) must be a multiple of 2, otherwise behaviour is not well-defined"
        );

        data_cache_writeback(&data.0);

        self.wait();

        self.set_dram_addr(k0_to_phys(data.0.as_ptr()).addr() as _);
        self.set_cart_addr(addr);
        self.set_bb_rd_len((len - 1) as _);

        self.wait();
    }

    pub fn bb_read<T: Default, const N: usize>(&mut self, addr: u32) -> [T; N] {
        let mut buf = Align8(from_fn(|_| Default::default()));

        self.bb_read_into(&mut buf, addr);

        buf.0
    }

    #[track_caller]
    pub fn bb_read_into<T>(&mut self, data: &mut Align8<[T]>, addr: u32) {
        let len = size_of_val(&data.0);

        assert!(
            addr % 2 == 0,
            "PI address ({addr:08X}) must be 2-byte aligned, otherwise behaviour is not well-defined"
        );
        assert!(
            len % 2 == 0,
            "Length ({len:X}) must be a multiple of 2, otherwise behaviour is not well-defined"
        );

        self.wait();

        self.set_dram_addr(k0_to_phys_mut(data.0.as_mut_ptr()).addr() as _);
        self.set_cart_addr(addr);
        self.set_bb_wr_len((len - 1) as _);

        self.wait();

        data_cache_invalidate(&data.0)
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

    pub fn bb_aes_ctrl(&self) -> u32 {
        unsafe { PI_BB_AES_CTRL.read_volatile() }
    }

    pub fn bb_allowed_io(&self) -> u32 {
        unsafe { PI_BB_ALLOWED_IO.read_volatile() }
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

    pub fn bb_ide_config(&self) -> u32 {
        unsafe { PI_BB_IDE_CONFIG.read_volatile() }
    }

    pub fn bb_ide_ctrl(&self) -> u32 {
        unsafe { PI_BB_IDE_CTRL.read_volatile() }
    }

    pub fn bb_nand_addr(&self) -> u32 {
        unsafe { PI_BB_NAND_ADDR.read_volatile() }
    }

    pub fn buffer0(&self, offset: u32) -> u32 {
        assert!(offset < 0x200);
        unsafe { io_ptr!(mut PI_BASE + 0x10000 + offset).read_volatile() }
    }

    pub fn buffer1(&self, offset: u32) -> u32 {
        assert!(offset < 0x200);
        unsafe { io_ptr!(mut PI_BASE + 0x10200 + offset).read_volatile() }
    }

    pub fn spare0(&self, offset: u32) -> u32 {
        assert!(offset < 0x10);
        unsafe { io_ptr!(mut PI_BASE + 0x10400 + offset).read_volatile() }
    }

    pub fn spare1(&self, offset: u32) -> u32 {
        assert!(offset < 0x10);
        unsafe { io_ptr!(mut PI_BASE + 0x10410 + offset).read_volatile() }
    }

    pub fn aes_expanded_key(&self, offset: u32) -> u32 {
        assert!(offset < 0xB0);
        unsafe { io_ptr!(mut PI_BASE + 0x10420 + offset).read_volatile() }
    }

    pub fn aes_iv(&self, offset: u32) -> u32 {
        assert!(offset < 0x10);
        unsafe { io_ptr!(mut PI_BASE + 0x104D0 + offset).read_volatile() }
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

    pub fn set_bb_aes_ctrl(&mut self, val: u32) {
        unsafe { PI_BB_AES_CTRL.write_volatile(val) }
    }

    pub fn set_bb_allowed_io(&mut self, val: u32) {
        unsafe { PI_BB_ALLOWED_IO.write_volatile(val) }
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

    pub fn set_bb_ide_config(&mut self, val: u32) {
        unsafe { PI_BB_IDE_CONFIG.write_volatile(val) }
    }

    pub fn set_bb_ide_ctrl(&mut self, val: u32) {
        unsafe { PI_BB_IDE_CTRL.write_volatile(val) }
    }

    pub fn set_bb_nand_addr(&mut self, val: u32) {
        unsafe { PI_BB_NAND_ADDR.write_volatile(val) }
    }

    pub fn set_buffer0(&mut self, offset: u32, val: u32) {
        assert!(offset < 0x200);
        unsafe { io_ptr!(mut PI_BASE + 0x10000 + offset).write_volatile(val) }
    }

    pub fn set_buffer1(&mut self, offset: u32, val: u32) {
        assert!(offset < 0x200);
        unsafe { io_ptr!(mut PI_BASE + 0x10200 + offset).write_volatile(val) }
    }

    pub fn set_spare0(&mut self, offset: u32, val: u32) {
        assert!(offset < 0x10);
        unsafe { io_ptr!(mut PI_BASE + 0x10400 + offset).write_volatile(val) }
    }

    pub fn set_spare1(&mut self, offset: u32, val: u32) {
        assert!(offset < 0x10);
        unsafe { io_ptr!(mut PI_BASE + 0x10410 + offset).write_volatile(val) }
    }

    pub fn set_aes_expanded_key(&mut self, offset: u32, val: u32) {
        assert!(offset < 0xB0);
        unsafe { io_ptr!(mut PI_BASE + 0x10420 + offset).write_volatile(val) }
    }

    pub fn set_aes_iv(&mut self, offset: u32, val: u32) {
        assert!(offset < 0x10);
        unsafe { io_ptr!(mut PI_BASE + 0x104D0 + offset).write_volatile(val) }
    }

    pub fn set_led(&mut self, val: LedValue) {
        let prev = self.bb_gpio() & !(1 << 1);
        let new = prev | (1 << 5) | ((val as u32) << 1);
        self.set_bb_gpio(new);
    }

    pub fn power_off(&mut self) {
        let prev = self.bb_gpio() & !(1 << 0);
        let new = prev | (1 << 4) | (0 << 0);
        self.set_bb_gpio(new);
    }

    #[cfg(feature = "sk")]
    pub fn power_on(&mut self) {
        let prev = self.bb_gpio();
        let new = prev | (1 << 4) | (1 << 0);
        self.set_bb_gpio(new);
    }
}

static mut PI: Pi = Pi::new();

pub fn pi() -> &'static mut Pi {
    unsafe { &mut PI }
}
