use core::ops::{Index, IndexMut};
use core::ptr::from_raw_parts_mut;

use crate::boot::is_bbplayer;
use crate::cop0::cop0;
use crate::mi::mi;
use crate::util::phys_to_k1_u32;
use crate::{data_cache_writeback, io_ptr};

const VI_BASE: u32 = 0x0440_0000;

const VI_CTRL: *mut u32 = io_ptr!(mut VI_BASE + 0x00);
const VI_ORIGIN: *mut u32 = io_ptr!(mut VI_BASE + 0x04);
const VI_WIDTH: *mut u32 = io_ptr!(mut VI_BASE + 0x08);
const VI_V_INTR: *mut u32 = io_ptr!(mut VI_BASE + 0x0C);
const VI_V_CURRENT: *mut u32 = io_ptr!(mut VI_BASE + 0x10);
const VI_BURST: *mut u32 = io_ptr!(mut VI_BASE + 0x14);
const VI_V_SYNC: *mut u32 = io_ptr!(mut VI_BASE + 0x18);
const VI_H_SYNC: *mut u32 = io_ptr!(mut VI_BASE + 0x1C);
const VI_H_SYNC_LEAP: *mut u32 = io_ptr!(mut VI_BASE + 0x20);
const VI_H_VIDEO: *mut u32 = io_ptr!(mut VI_BASE + 0x24);
const VI_V_VIDEO: *mut u32 = io_ptr!(mut VI_BASE + 0x28);
const VI_V_BURST: *mut u32 = io_ptr!(mut VI_BASE + 0x2C);
const VI_X_SCALE: *mut u32 = io_ptr!(mut VI_BASE + 0x30);
const VI_Y_SCALE: *mut u32 = io_ptr!(mut VI_BASE + 0x34);
const VI_TEST_ADDR: *mut u32 = io_ptr!(mut VI_BASE + 0x38);
const VI_STAGED_DATA: *mut u32 = io_ptr!(mut VI_BASE + 0x3C);

mod burst;
mod ctrl;
mod h_sync;
mod h_sync_leap;
mod scale;
mod video;

use burst::*;
use ctrl::*;
use h_sync::*;
use h_sync_leap::*;
use scale::*;
use video::*;

#[derive(Clone, Copy)]
struct FixedPoint(u16);

impl FixedPoint {
    const fn new(integer: u8, fraction: u16) -> Self {
        Self(((integer as u16 & 0x03) << 10) | (fraction & 0x03FF))
    }

    const fn to_u16(self) -> u16 {
        self.0
    }
}

pub struct Vi {
    #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
    next_framebuffer: bool,
}

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;

#[repr(align(8))]
struct Framebuffer<const W: usize, const H: usize, const N: usize, T>([[T; W * H]; N])
where
    [(); W * H]:;

impl<const W: usize, const H: usize, const N: usize, T, Idx> Index<Idx> for Framebuffer<W, H, N, T>
where
    [[T; W * H]; N]: Index<Idx, Output = [T; W * H]>,
{
    type Output = [T; W * H];

    fn index(&self, index: Idx) -> &Self::Output {
        self.0.index(index)
    }
}

impl<const W: usize, const H: usize, const N: usize, T, Idx> IndexMut<Idx>
    for Framebuffer<W, H, N, T>
where
    [[T; W * H]; N]: IndexMut<Idx, Output = [T; W * H]>,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

#[cfg_attr(feature = "sk_vi", link_section = ".dram")]
static mut FRAMEBUFFER: Framebuffer<WIDTH, HEIGHT, 2, u32> =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

pub enum Mode {
    PAL50,
    PAL60,
    NTSC,
}

impl Vi {
    const fn new() -> Self {
        Self {
            #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
            next_framebuffer: false,
        }
    }

    #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
    pub fn get_next_framebuffer(&self) -> &'static mut [u32; WIDTH * HEIGHT] {
        // godawful codegen on this for some reason
        unsafe { &mut FRAMEBUFFER[self.next_framebuffer as usize] }
    }

    #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
    pub fn next_framebuffer(&mut self) {
        data_cache_writeback(self.get_next_framebuffer());
        self.set_origin(self.get_next_framebuffer().as_ptr().addr() as _);
        self.next_framebuffer = !self.next_framebuffer;
    }

    #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
    pub fn clear_framebuffer(&mut self) {
        self.get_next_framebuffer().fill(0)
    }

    #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
    pub fn blank(&mut self) {
        self.set_h_video(0);
    }

    #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
    pub fn wait_vsync(&self) {
        while (self.v_current() >> 1) & 0x1FF != 1 {}
        while (self.v_current() >> 1) & 0x1FF == 1 {}
    }

    pub fn ctrl(&self) -> u32 {
        unsafe { VI_CTRL.read_volatile() }
    }

    pub fn origin(&self) -> u32 {
        unsafe { VI_ORIGIN.read_volatile() }
    }

    pub fn width(&self) -> u32 {
        unsafe { VI_WIDTH.read_volatile() }
    }

    pub fn v_intr(&self) -> u32 {
        unsafe { VI_V_INTR.read_volatile() }
    }

    pub fn v_current(&self) -> u32 {
        unsafe { VI_V_CURRENT.read_volatile() }
    }

    pub fn burst(&self) -> u32 {
        unsafe { VI_BURST.read_volatile() }
    }

    pub fn v_sync(&self) -> u32 {
        unsafe { VI_V_SYNC.read_volatile() }
    }

    pub fn h_sync(&self) -> u32 {
        unsafe { VI_H_SYNC.read_volatile() }
    }

    pub fn h_sync_leap(&self) -> u32 {
        unsafe { VI_H_SYNC_LEAP.read_volatile() }
    }

    pub fn h_video(&self) -> u32 {
        unsafe { VI_H_VIDEO.read_volatile() }
    }

    pub fn v_video(&self) -> u32 {
        unsafe { VI_V_VIDEO.read_volatile() }
    }

    pub fn v_burst(&self) -> u32 {
        unsafe { VI_V_BURST.read_volatile() }
    }

    pub fn x_scale(&self) -> u32 {
        unsafe { VI_X_SCALE.read_volatile() }
    }

    pub fn y_scale(&self) -> u32 {
        unsafe { VI_Y_SCALE.read_volatile() }
    }

    pub fn test_addr(&self) -> u32 {
        unsafe { VI_TEST_ADDR.read_volatile() }
    }

    pub fn staged_data(&self) -> u32 {
        unsafe { VI_STAGED_DATA.read_volatile() }
    }

    pub fn set_ctrl(&mut self, val: u32) {
        unsafe { VI_CTRL.write_volatile(val) }
    }

    pub fn set_origin(&mut self, val: u32) {
        assert!(val & 7 == 0, "Misaligned framebuffer");
        unsafe { VI_ORIGIN.write_volatile(val) }
    }

    pub fn set_width(&mut self, val: u32) {
        unsafe { VI_WIDTH.write_volatile(val) }
    }

    pub fn set_v_intr(&mut self, val: u32) {
        unsafe { VI_V_INTR.write_volatile(val) }
    }

    pub fn set_v_current(&mut self, val: u32) {
        unsafe { VI_V_CURRENT.write_volatile(val) }
    }

    pub fn set_burst(&mut self, val: u32) {
        unsafe { VI_BURST.write_volatile(val) }
    }

    pub fn set_v_sync(&mut self, val: u32) {
        unsafe { VI_V_SYNC.write_volatile(val) }
    }

    pub fn set_h_sync(&mut self, val: u32) {
        unsafe { VI_H_SYNC.write_volatile(val) }
    }

    pub fn set_h_sync_leap(&mut self, val: u32) {
        unsafe { VI_H_SYNC_LEAP.write_volatile(val) }
    }

    pub fn set_h_video(&mut self, val: u32) {
        unsafe { VI_H_VIDEO.write_volatile(val) }
    }

    pub fn set_v_video(&mut self, val: u32) {
        unsafe { VI_V_VIDEO.write_volatile(val) }
    }

    pub fn set_v_burst(&mut self, val: u32) {
        unsafe { VI_V_BURST.write_volatile(val) }
    }

    pub fn set_x_scale(&mut self, val: u32) {
        unsafe { VI_X_SCALE.write_volatile(val) }
    }

    pub fn set_y_scale(&mut self, val: u32) {
        unsafe { VI_Y_SCALE.write_volatile(val) }
    }

    pub fn set_test_addr(&mut self, val: u32) {
        unsafe { VI_TEST_ADDR.write_volatile(val) }
    }

    pub fn set_staged_data(&mut self, val: u32) {
        unsafe { VI_STAGED_DATA.write_volatile(val) }
    }

    fn calibrate_init(&mut self) {
        let save_ctrl = self.ctrl();

        self.set_ctrl(Ctrl::test_mode(true) | Ctrl::kill_we(true));

        self.set_test_addr(0);
        self.set_staged_data(0x43210123);

        self.set_test_addr(1);
        self.set_staged_data(0);

        self.set_test_addr(2);
        self.set_staged_data(0);

        self.set_test_addr(3);
        self.set_staged_data(0);

        self.set_ctrl(save_ctrl);
    }

    #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
    fn calibrate_check(&mut self) -> bool {
        let save_ctrl = self.ctrl();

        self.set_ctrl(Ctrl::test_mode(true) | Ctrl::kill_we(true));

        self.set_test_addr(0);
        let data = self.staged_data();

        self.set_ctrl(save_ctrl);

        data == 0x43210123
    }

    pub fn init_calibrate(&mut self) {
        let cop0 = cop0();

        let mi = mi();

        cop0.disable_interrupts();

        self.clear_framebuffer();

        loop {
            self.set_ctrl(0);

            cop0.delay(10);

            let temp = mi.unknown(0x30) & !(1 << 25);
            mi.set_unknown(0x30, temp);
            cop0.delay(1);
            mi.set_unknown(0x30, temp | (1 << 25));

            self.set_origin(self.get_next_framebuffer().as_ptr().addr() as _);
            self.set_width(WIDTH as _);
            self.set_v_intr(2);

            self.set_burst(
                Burst::burst_start(62)
                    | Burst::vsync_width(5)
                    | Burst::burst_width(34)
                    | Burst::hsync_width(57),
            );
            self.set_v_sync(525);
            self.set_h_sync(HSync::leap(0) | HSync::h_sync(3093));
            self.set_h_sync_leap(HSyncLeap::leap_a(3093) | HSyncLeap::leap_b(3093));
            self.set_h_video(Video::start(108) | Video::end(748));
            self.set_v_video(Video::start(37) | Video::end(511));
            self.set_v_burst(Video::start(14) | Video::end(516));
            self.set_x_scale(
                Scale::offset(FixedPoint::new(0, 0)) | Scale::scale(FixedPoint::new(0, 512)),
            );
            self.set_y_scale(
                Scale::offset(FixedPoint::new(0, 0)) | Scale::scale(FixedPoint::new(1, 0)),
            );

            self.calibrate_init();

            self.set_v_current(0);

            self.set_ctrl(
                Ctrl::dedither_enable(true)
                    | Ctrl::pixel_advance(if is_bbplayer() { 1 } else { 3 })
                    | Ctrl::kill_we(false)
                    | Ctrl::aa_mode(AAMode::None)
                    | Ctrl::test_mode(false)
                    | Ctrl::serrate(false)
                    | Ctrl::vbus_clock_enable(false)
                    | Ctrl::divot_enable(false)
                    | Ctrl::gamma_enable(true)
                    | Ctrl::gamma_dither_enable(true)
                    | Ctrl::pixel_size(PixelSize::Rgba8),
            );

            while self.v_current() < 48 {}

            if !self.calibrate_check() {
                break;
            }
        }

        self.set_h_video(0);

        cop0.enable_interrupts();
    }

    #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
    pub fn init(&mut self, mode: Mode) {
        self.set_ctrl(
            Ctrl::dedither_enable(true)
                | Ctrl::pixel_advance(if is_bbplayer() { 1 } else { 3 })
                | Ctrl::kill_we(false)
                | Ctrl::aa_mode(AAMode::None)
                | Ctrl::test_mode(false)
                | Ctrl::serrate(false)
                | Ctrl::vbus_clock_enable(false)
                | Ctrl::divot_enable(false)
                | Ctrl::gamma_enable(true)
                | Ctrl::gamma_dither_enable(true)
                | Ctrl::pixel_size(PixelSize::Rgba8),
        );

        self.set_origin(self.get_next_framebuffer().as_ptr().addr() as _);
        self.set_width(WIDTH as _);
        self.set_v_intr(2);

        match mode {
            Mode::PAL50 => todo!(),
            Mode::PAL60 => {
                self.set_burst(
                    Burst::burst_start(69)
                        | Burst::vsync_width(4)
                        | Burst::burst_width(30)
                        | Burst::hsync_width(58),
                );
                self.set_v_sync(519);
                self.set_h_sync(HSync::leap(23) | HSync::h_sync(3177));
                self.set_h_sync_leap(HSyncLeap::leap_a(3183) | HSyncLeap::leap_b(3181));
                self.set_h_video(Video::start(128) | Video::end(768));
            }
            Mode::NTSC => {
                self.set_burst(
                    Burst::burst_start(62)
                        | Burst::vsync_width(5)
                        | Burst::burst_width(34)
                        | Burst::hsync_width(57),
                );
                self.set_v_sync(525);
                self.set_h_sync(HSync::leap(0) | HSync::h_sync(3093));
                self.set_h_sync_leap(HSyncLeap::leap_a(3093) | HSyncLeap::leap_b(3093));
                self.set_h_video(Video::start(108) | Video::end(748));
            }
        }

        self.set_v_video(Video::start(37) | Video::end(511));
        self.set_v_burst(Video::start(14) | Video::end(516));
        self.set_x_scale(
            Scale::offset(FixedPoint::new(0, 0)) | Scale::scale(FixedPoint::new(0, 512)),
        );
        self.set_y_scale(
            Scale::offset(FixedPoint::new(0, 0)) | Scale::scale(FixedPoint::new(1, 0)),
        );

        self.next_framebuffer();
    }

    #[cfg(any(not(feature = "sk"), feature = "sk_vi"))]
    pub fn pll_init(&mut self) {
        let cop0 = cop0();

        let mi = mi();

        cop0.disable_interrupts();

        self.set_ctrl(0); // clear VI interrupt
        unsafe {
            from_raw_parts_mut::<u32>(phys_to_k1_u32(0x0450000C) as *mut (), ()).write_volatile(0);
            // clear AI interrupt
        }

        cop0.delay(50);

        let avctrl = mi.unknown(0x30) & !(1 << 25);
        mi.set_unknown(0x30, avctrl);

        cop0.delay(50);

        mi.set_unknown(0x30, avctrl | (1 << 0));

        cop0.delay(50);

        let avctrl = 0x194244; // NTSC

        mi.set_unknown(0x30, avctrl | (1 << 23) | (1 << 0));
        mi.unknown(0x30);

        cop0.delay(2);

        mi.set_unknown(0x30, avctrl);
        mi.unknown(0x30);

        cop0.delay(1000);

        mi.set_unknown(0x30, avctrl | (1 << 25));
        mi.unknown(0x30);

        cop0.enable_interrupts();
    }
}

static mut VI: Vi = Vi::new();

pub fn vi() -> &'static mut Vi {
    unsafe { &mut VI }
}
