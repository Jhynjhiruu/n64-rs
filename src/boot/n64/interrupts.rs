#![allow(clippy::unusual_byte_groupings)]

use core::{
    arch::naked_asm,
    ffi::c_void,
    hint::unreachable_unchecked,
    mem::size_of,
    ptr::{addr_of, from_raw_parts, from_raw_parts_mut},
};

use crate::{
    cop0::{cop0, Cause, DiagStatus, ExceptionCode, ExecutionMode, Status}, data_cache_writeback, instruction_cache_invalidate, mi::{mi, BBInterrupt, Interrupt}, pi::pi, si::si, types::Align8, vi::vi
};

use super::is_bbplayer;

pub type IntFn = fn() -> bool;

#[derive(Debug)]
pub struct InterruptManager {
    sp: bool,
    si: bool,
    ai: bool,
    vi: bool,
    pi: bool,
    dp: bool,
    flash: bool,
    aes: bool,
    ide: bool,
    pi_err: bool,
    usb0: bool,
    usb1: bool,
    btn: bool,
    md: bool,

    prenmi: bool,
    tmr: bool,

    sp_fn: Option<IntFn>,
    si_fn: Option<IntFn>,
    ai_fn: Option<IntFn>,
    vi_fn: Option<IntFn>,
    pi_fn: Option<IntFn>,
    dp_fn: Option<IntFn>,
    flash_fn: Option<IntFn>,
    aes_fn: Option<IntFn>,
    ide_fn: Option<IntFn>,
    pi_err_fn: Option<IntFn>,
    usb0_fn: Option<IntFn>,
    usb1_fn: Option<IntFn>,
    // there is no button interrupt
    //btn_fn: Option<IntFn>,
    md_fn: Option<IntFn>,

    prenmi_fn: Option<IntFn>,
    tmr_fn: Option<IntFn>,
}

impl InterruptManager {
    pub const fn new() -> Self {
        Self {
            sp: false,
            si: false,
            ai: false,
            vi: false,
            pi: false,
            dp: false,
            flash: false,
            aes: false,
            ide: false,
            pi_err: false,
            usb0: false,
            usb1: false,
            btn: false,
            md: false,
            prenmi: false,
            tmr: false,
            sp_fn: None,
            si_fn: None,
            ai_fn: None,
            vi_fn: None,
            pi_fn: None,
            dp_fn: None,
            flash_fn: None,
            aes_fn: None,
            ide_fn: None,
            pi_err_fn: None,
            usb0_fn: None,
            usb1_fn: None,
            //btn_fn: None,
            md_fn: None,
            prenmi_fn: None,
            tmr_fn: None,
        }
    }

    fn update(&mut self) {
        let mi = mi();
        let cop0 = cop0();

        cop0.disable_interrupts();

        if is_bbplayer() {
            let mask = ((1 << ((self.sp & self.sp_fn.is_some()) as u32)) << (0 * 2))
                | ((1 << ((self.si & self.si_fn.is_some()) as u32)) << (1 * 2))
                | ((1 << ((self.ai & self.ai_fn.is_some()) as u32)) << (2 * 2))
                | ((1 << ((self.vi & self.vi_fn.is_some()) as u32)) << (3 * 2))
                | ((1 << ((self.pi & self.pi_fn.is_some()) as u32)) << (4 * 2))
                | ((1 << ((self.dp & self.dp_fn.is_some()) as u32)) << (5 * 2))
                | ((1 << ((self.flash & self.flash_fn.is_some()) as u32)) << (6 * 2))
                | ((1 << ((self.aes & self.aes_fn.is_some()) as u32)) << (7 * 2))
                | ((1 << ((self.ide & self.ide_fn.is_some()) as u32)) << (8 * 2))
                | ((1 << ((self.pi_err & self.pi_err_fn.is_some()) as u32)) << (9 * 2))
                | ((1 << ((self.usb0 & self.usb0_fn.is_some()) as u32)) << (10 * 2))
                | ((1 << ((self.usb1 & self.usb1_fn.is_some()) as u32)) << (11 * 2))
                | ((1 << ((self.btn/* & self.btn_fn.is_some()*/) as u32)) << (12 * 2))
                | ((1 << ((self.md & self.md_fn.is_some()) as u32)) << (13 * 2));

            mi.set_bb_mask(mask);

            let rcp_interrupts = (mask & 0b00_00_00_00__00_00_00_00__00_00_10_10__10_10_10_10) != 0;
            let bb_interrupts = (mask & 0b00_00_10_10__10_10_10_10__10_10_00_00__00_00_00_00) != 0;
            let prenmi_interrupt = self.prenmi & self.prenmi_fn.is_some();
            let tmr_interrupt = self.tmr & self.tmr_fn.is_some();

            let status = cop0.status()
                & !Status::interrupt_mask(Status::HW0 | Status::HW1 | Status::HW2 | Status::TMR);
            cop0.set_status(
                status
                    | Status::interrupt_mask(
                        if rcp_interrupts { Status::HW0 } else { 0 }
                            | if bb_interrupts { Status::HW1 } else { 0 }
                            | if prenmi_interrupt { Status::HW2 } else { 0 }
                            | if tmr_interrupt { Status::TMR } else { 0 },
                    ),
            )
        } else {
            let mask = ((1 << ((self.sp & self.sp_fn.is_some()) as u32)) << (0 * 2))
                | ((1 << ((self.si & self.si_fn.is_some()) as u32)) << (1 * 2))
                | ((1 << ((self.ai & self.ai_fn.is_some()) as u32)) << (2 * 2))
                | ((1 << ((self.vi & self.vi_fn.is_some()) as u32)) << (3 * 2))
                | ((1 << ((self.pi & self.pi_fn.is_some()) as u32)) << (4 * 2))
                | ((1 << ((self.dp & self.dp_fn.is_some()) as u32)) << (5 * 2));

            mi.set_mask(mask);

            let rcp_interrupts = (mask & 0b00_00_00_00__00_00_00_00__00_00_10_10__10_10_10_10) != 0;
            let prenmi_interrupt = self.prenmi & self.prenmi_fn.is_some();
            let tmr_interrupt = self.tmr & self.tmr_fn.is_some();

            let status =
                cop0.status() & !Status::interrupt_mask(Status::HW0 | Status::HW2 | Status::TMR);
            cop0.set_status(
                status
                    | Status::interrupt_mask(
                        if rcp_interrupts { Status::HW0 } else { 0 }
                            | if prenmi_interrupt { Status::HW2 } else { 0 }
                            | if tmr_interrupt { Status::TMR } else { 0 },
                    ),
            );
        }

        cop0.enable_interrupts();
    }

    pub fn set_sp(&mut self, value: bool) {
        self.sp = value;
        self.update();
    }

    pub fn set_si(&mut self, value: bool) {
        self.si = value;
        self.update();
    }

    pub fn set_ai(&mut self, value: bool) {
        self.ai = value;
        self.update();
    }

    pub fn set_vi(&mut self, value: bool) {
        self.vi = value;
        self.update();
    }

    pub fn set_pi(&mut self, value: bool) {
        self.pi = value;
        self.update();
    }

    pub fn set_dp(&mut self, value: bool) {
        self.dp = value;
        self.update();
    }

    pub fn set_flash(&mut self, value: bool) {
        self.flash = value;
        self.update();
    }

    pub fn set_aes(&mut self, value: bool) {
        self.aes = value;
        self.update();
    }

    pub fn set_ide(&mut self, value: bool) {
        self.ide = value;
        self.update();
    }

    pub fn set_pi_err(&mut self, value: bool) {
        self.pi_err = value;
        self.update();
    }

    pub fn set_usb0(&mut self, value: bool) {
        self.usb0 = value;
        self.update();
    }

    pub fn set_usb1(&mut self, value: bool) {
        self.usb1 = value;
        self.update();
    }

    pub fn set_btn(&mut self, value: bool) {
        self.btn = value;
        self.update();
    }

    pub fn set_md(&mut self, value: bool) {
        self.md = value;
        self.update();
    }

    pub fn set_prenmi(&mut self, value: bool) {
        self.prenmi = value;
        self.update();
    }

    pub fn set_tmr(&mut self, value: bool) {
        self.tmr = value;
        self.update();
    }

    pub fn set_sp_fn(&mut self, value: Option<IntFn>) {
        self.sp_fn = value;
        self.update();
    }

    pub fn set_si_fn(&mut self, value: Option<IntFn>) {
        self.si_fn = value;
        self.update();
    }

    pub fn set_ai_fn(&mut self, value: Option<IntFn>) {
        self.ai_fn = value;
        self.update();
    }

    pub fn set_vi_fn(&mut self, value: Option<IntFn>) {
        self.vi_fn = value;
        self.update();
    }

    pub fn set_pi_fn(&mut self, value: Option<IntFn>) {
        self.pi_fn = value;
        self.update();
    }

    pub fn set_dp_fn(&mut self, value: Option<IntFn>) {
        self.dp_fn = value;
        self.update();
    }

    pub fn set_flash_fn(&mut self, value: Option<IntFn>) {
        self.flash_fn = value;
        self.update();
    }

    pub fn set_aes_fn(&mut self, value: Option<IntFn>) {
        self.aes_fn = value;
        self.update();
    }

    pub fn set_ide_fn(&mut self, value: Option<IntFn>) {
        self.ide_fn = value;
        self.update();
    }

    pub fn set_pi_err_fn(&mut self, value: Option<IntFn>) {
        self.pi_err_fn = value;
        self.update();
    }

    pub fn set_usb0_fn(&mut self, value: Option<IntFn>) {
        self.usb0_fn = value;
        self.update();
    }

    pub fn set_usb1_fn(&mut self, value: Option<IntFn>) {
        self.usb1_fn = value;
        self.update();
    }

    // there is no button interrupt
    /*pub fn set_btn_fn(&mut self, value: Option<IntFn>) {
        self.btn_fn = value;
        self.update();
    }*/

    pub fn set_md_fn(&mut self, value: Option<IntFn>) {
        self.md_fn = value;
        self.update();
    }

    pub fn set_prenmi_fn(&mut self, value: Option<IntFn>) {
        self.prenmi_fn = value;
        self.update();
    }

    pub fn set_tmr_fn(&mut self, value: Option<IntFn>) {
        self.tmr_fn = value;
        self.update();
    }
}

static mut IM: InterruptManager = InterruptManager::new();

pub fn im() -> &'static mut InterruptManager {
    unsafe { &mut IM }
}

fn black_screen() {
    let vi = vi();
    vi.blank();
}

extern "C" fn int_handler() {
    let cop0 = cop0();

    let cause = cop0.cause();

    if cause & Cause::EC == Cause::exception_code(ExceptionCode::Interrupt) {
        // interrupt

        // here's where libdragon disables the FPU, but we're not using the FPU (yet?)

        // we don't use the ticks, either

        if cause & Cause::interrupt_pending(Cause::HW2) != 0 {
            // prenmi
            let im = im();
            im.set_prenmi(false);

            #[cfg(not(debug_assertions))]
            let fun = im.prenmi_fn.unwrap();
            #[cfg(debug_assertions)]
            let fun = im
                .prenmi_fn
                .expect("PreNMI interrupt is enabled, so there should be a handler installed");

            if fun() {
                // can't ack prenmi, so disable it
                im.set_prenmi(false);
            }
        }

        if cause & Cause::interrupt_pending(Cause::TMR) != 0 {
            // timer
            let im = im();

            #[cfg(not(debug_assertions))]
            let fun = im.tmr_fn.unwrap();
            #[cfg(debug_assertions)]
            let fun = im
                .tmr_fn
                .expect("Timer interrupt is enabled, so there should be a handler installed");

            if fun() {
                // ack timer by setting compare to any value
                // we use itself to avoid clobbering it if it's needed
                cop0.set_compare(cop0.compare());
            }
        }

        macro_rules! call_handler {
            ($s:expr, $name:literal, $ex:path, $fn:expr, $b:block) => {
                #[cfg(not(debug_assertions))]
                let fun = $fn.unwrap();
                #[cfg(debug_assertions)]
                let fun = $fn.expect(concat!(
                    $name,
                    " interrupt is enabled, so there should be a handler installed"
                ));

                if $s & $ex(true) != 0 && fun() {
                    $b
                }
            };
        }

        if cause & Cause::interrupt_pending(Cause::HW1) != 0 {
            // bb
            let im = im();

            let mi = mi();

            let status = mi.bb_interrupt() & mi.bb_mask();

            call_handler!(status, "Flash", BBInterrupt::flash, im.flash_fn, {
                pi().set_bb_nand_ctrl(0);
            });

            call_handler!(status, "AES", BBInterrupt::aes, im.aes_fn, {
                pi().set_bb_aes_ctrl(0);
            });

            call_handler!(status, "IDE", BBInterrupt::ide, im.ide_fn, {
                panic!("Unhandled BB interrupt: IDE");
            });

            call_handler!(status, "PI error", BBInterrupt::pi_err, im.pi_err_fn, {
                panic!("Unhandled BB interrupt: PI error");
            });

            call_handler!(status, "USB0", BBInterrupt::usb0, im.usb0_fn, {
                panic!("USB0 interrupt should be acknowledged in the handler");
            });

            call_handler!(status, "USB1", BBInterrupt::usb1, im.usb1_fn, {
                panic!("USB1 interrupt should be acknowledged in the handler");
            });

            // there is no button interrupt
            /*call_handler!(status, "Button", BBInterrupt::btn, im.btn_fn, {
                panic!("Unhandled BB interrupt: Button");
            });*/

            call_handler!(status, "Module", BBInterrupt::md, im.md_fn, {
                mi.set_bb_interrupt(BBInterrupt::md(true));
            });
        }

        if cause & Cause::interrupt_pending(Cause::HW0) != 0 {
            // rcp
            let im = im();

            let mi = mi();

            let status = mi.interrupt() & mi.mask();

            call_handler!(status, "SP", Interrupt::sp, im.sp_fn, {
                panic!("Unhandled RCP interrupt: SP");
            });

            call_handler!(status, "SI", Interrupt::si, im.si_fn, {
                si().set_status(0);
            });

            call_handler!(status, "AI", Interrupt::ai, im.ai_fn, {
                panic!("Unhandled RCP interrupt: AI");
            });

            call_handler!(status, "VI", Interrupt::vi, im.vi_fn, {
                vi().set_v_current(vi().v_current());
            });

            call_handler!(status, "PI", Interrupt::pi, im.pi_fn, {
                pi().set_status(1 << 1);
            });

            call_handler!(status, "DP", Interrupt::dp, im.dp_fn, {
                mi.set_mode(1 << 11);
            });
        }
    } else if cause & Cause::EC == Cause::exception_code(ExceptionCode::Watch) {
        if cop0.error_epc() < 0x80000000 {
            cop0.clear_watch();
            cop0.set_compare(cop0.epc());
        } else {
            panic!(
                "Watch exception\nCompare: {:08X}\nEPC: {:08X}\nWatch: {:08X}\nErrorEPC: {:08X}",
                cop0.compare(),
                cop0.epc() + 280,
                cop0.watch_lo(),
                cop0.error_epc(),
            );
        }
    } else {
        // exception

        panic!(
            "Exceptions not yet handled\nCause: {:08X}\nErrorEPC: {:08X}\nEPC: {:08X}",
            cop0.cause(),
            cop0.error_epc(),
            cop0.epc()
        );
    }
}

extern "C" fn update_interrupts() {
    im().update();
}

const ABI_REQUIRED_SPACE: usize = /*4 * size_of::<u32>()*/ 0; // required in o32, but we're using n32
const NUM_GPRS: usize = 32;
const NUM_HILO: usize = 2;
const NUM_COP0: usize = 2;
const EXCEPTION_STACK_DIFF: usize = ABI_REQUIRED_SPACE
    + (size_of::<u64>() * NUM_GPRS)
    + (size_of::<u64>() * NUM_HILO)
    + (size_of::<u32>() * NUM_COP0);
const GPRS_START: usize = ABI_REQUIRED_SPACE;
const HILO_START: usize = GPRS_START + (size_of::<u64>() * NUM_GPRS);
const COP0_START: usize = HILO_START + (size_of::<u64>() * NUM_HILO);

#[link_section = ".text"]
#[naked]
unsafe extern "C" fn _int_handler() {
    naked_asm!(
        ".set noreorder                  ",
        "  addiu $sp, -{diff}            ",
        "                                ",
        ".set noat                       ",
        "  sd $1,  ({gprs} +  1 * 8)($sp)", // at
        ".set at                         ",
        "  sd $2,  ({gprs} +  2 * 8)($sp)", // v0
        "  sd $3,  ({gprs} +  3 * 8)($sp)", // v1
        "  sd $4,  ({gprs} +  4 * 8)($sp)", // a0
        "  sd $5,  ({gprs} +  5 * 8)($sp)", // a1
        "  sd $6,  ({gprs} +  6 * 8)($sp)", // a2
        "  sd $7,  ({gprs} +  7 * 8)($sp)", // a3
        "  sd $8,  ({gprs} +  8 * 8)($sp)", // a4
        "  sd $9,  ({gprs} +  9 * 8)($sp)", // a5
        "  sd $10, ({gprs} + 10 * 8)($sp)", // a6
        "  sd $11, ({gprs} + 11 * 8)($sp)", // a7
        "  sd $12, ({gprs} + 12 * 8)($sp)", // t4
        "  sd $13, ({gprs} + 13 * 8)($sp)", // t5
        "  sd $14, ({gprs} + 14 * 8)($sp)", // t6
        "  sd $15, ({gprs} + 15 * 8)($sp)", // t7
        "  sd $16, ({gprs} + 16 * 8)($sp)", // s0
        "  sd $17, ({gprs} + 17 * 8)($sp)", // s1
        "  sd $18, ({gprs} + 18 * 8)($sp)", // s2
        "  sd $19, ({gprs} + 19 * 8)($sp)", // s3
        "  sd $20, ({gprs} + 20 * 8)($sp)", // s4
        "  sd $21, ({gprs} + 21 * 8)($sp)", // s5
        "  sd $22, ({gprs} + 22 * 8)($sp)", // s6
        "  sd $23, ({gprs} + 23 * 8)($sp)", // s7
        "  sd $24, ({gprs} + 24 * 8)($sp)", // t8
        "  sd $25, ({gprs} + 25 * 8)($sp)", // t9
        "  sd $26, ({gprs} + 26 * 8)($sp)", // kt0
        "  sd $27, ({gprs} + 27 * 8)($sp)", // kt1
        "  sd $28, ({gprs} + 28 * 8)($sp)", // gp
        "  sd $29, ({gprs} + 29 * 8)($sp)", // sp
        "  sd $30, ({gprs} + 30 * 8)($sp)", // s8
        "  sd $31, ({gprs} + 31 * 8)($sp)", // ra
        "                                ",
        "  mflo $k0                      ",
        "  mfhi $k1                      ",
        "  sd $k0, ({hilo} +  0 * 8)($sp)",
        "  sd $k1, ({hilo} +  1 * 8)($sp)",
        "                                ",
        "  mfc0 $k0, $14                 ", // epc
        "  mfc0 $k1, $12                 ", // sr
        "  sw $k0, ({cop0} +  0 * 4)($sp)",
        "                                ",
        "  andi $k0, $k1, {ints}         ",
        "  and $k1, $k1, ~({ints_exl})   ",
        "  mtc0 $k1, $12                 ", // sr
        "                                ",
        "  sw $k0, ({cop0} +  1 * 4)($sp)",
        "                                ",
        "  jal {handler}                 ",
        "   nop                          ",
        "                                ",
        "  lw $k1, ({cop0} +  1 * 4)($sp)",
        "  mfc0 $k0, $12                 ", // sr
        "  or $k1, $k1, $k0              ",
        "  mtc0 $k1, $12                 ", // sr
        "                                ",
        "  lw $k0, ({cop0} +  0 * 4)($sp)",
        "  mtc0 $k0, $14                 ", // epc
        "                                ",
        "  ld $k1, ({hilo} +  1 * 8)($sp)",
        "  ld $k0, ({hilo} +  0 * 8)($sp)",
        "  mthi $k1                      ",
        "  mtlo $k0                      ",
        "                                ",
        "  ld $31, ({gprs} + 31 * 8)($sp)", // ra
        "  ld $30, ({gprs} + 30 * 8)($sp)", // s8
        "  ld $29, ({gprs} + 29 * 8)($sp)", // sp
        "  ld $28, ({gprs} + 28 * 8)($sp)", // gp
        "  ld $27, ({gprs} + 27 * 8)($sp)", // kt1
        "  ld $26, ({gprs} + 26 * 8)($sp)", // kt0
        "  ld $25, ({gprs} + 25 * 8)($sp)", // t9
        "  ld $24, ({gprs} + 24 * 8)($sp)", // t8
        "  ld $23, ({gprs} + 23 * 8)($sp)", // s7
        "  ld $22, ({gprs} + 22 * 8)($sp)", // s6
        "  ld $21, ({gprs} + 21 * 8)($sp)", // s5
        "  ld $20, ({gprs} + 20 * 8)($sp)", // s4
        "  ld $19, ({gprs} + 19 * 8)($sp)", // s3
        "  ld $18, ({gprs} + 18 * 8)($sp)", // s2
        "  ld $17, ({gprs} + 17 * 8)($sp)", // s1
        "  ld $16, ({gprs} + 16 * 8)($sp)", // s0
        "  ld $15, ({gprs} + 15 * 8)($sp)", // t7
        "  ld $14, ({gprs} + 14 * 8)($sp)", // t6
        "  ld $13, ({gprs} + 13 * 8)($sp)", // t5
        "  ld $12, ({gprs} + 12 * 8)($sp)", // t4
        "  ld $11, ({gprs} + 11 * 8)($sp)", // a7
        "  ld $10, ({gprs} + 10 * 8)($sp)", // a6
        "  ld $9,  ({gprs} +  9 * 8)($sp)", // a5
        "  ld $8,  ({gprs} +  8 * 8)($sp)", // a4
        "  ld $7,  ({gprs} +  7 * 8)($sp)", // a3
        "  ld $6,  ({gprs} +  6 * 8)($sp)", // a2
        "  ld $5,  ({gprs} +  5 * 8)($sp)", // a1
        "  ld $4,  ({gprs} +  4 * 8)($sp)", // a0
        "  ld $3,  ({gprs} +  3 * 8)($sp)", // v1
        "  ld $2,  ({gprs} +  2 * 8)($sp)", // v0
        ".set noat                       ",
        "  ld $1,  ({gprs} +  1 * 8)($sp)", // at
        ".set at                         ",
        "                                ",
        "  addiu $sp, {diff}             ",
        "                                ",
        "  eret                          ",
        "   nop                          ",
        ".set reorder                    ",
        diff = const EXCEPTION_STACK_DIFF,
        gprs = const GPRS_START,
        hilo = const HILO_START,
        cop0 = const COP0_START,
        ints = const Status::interrupt_enable(true),
        ints_exl = const (Status::interrupt_enable(true) | Status::exception(true)),
        handler = sym int_handler,
        //update_interrupts = sym update_interrupts,
    )
}

#[link_section = ".int_handler"]
#[naked]
#[no_mangle]
unsafe extern "C" fn int_handler_entry() -> ! {
    naked_asm!(
        "  .set noreorder",
        "    j {handler} ",
        "     nop        ",
        "  .set reorder  ",
        handler = sym _int_handler,
    )
}

extern "C" {
    static int_handler_ROM_START: c_void;
    static int_handler_ROM_END: c_void;
}

pub unsafe fn setup_ints() {
    let cop0 = cop0();
    // clear pending interrupt causes
    //cop0.set_cause(cop0.cause() & (Cause::branch_delay(true) | Cause::CE | Cause::EC));
    // enable interrupts and mask pre-NMI

    cop0.set_status(
        Status::coprocessor_usable(0)
            | Status::coprocessor_usable(1)
            | Status::reduced_power(false)
            | Status::more_float_registers(true)
            | Status::reverse_endian(false)
            | Status::diagnostic_status(
                DiagStatus::instruction_trace_support(false)
                    | DiagStatus::bootstrap_exception_vectors(false)
                    | DiagStatus::tlb_shutdown(false)
                    | DiagStatus::soft_reset(false)
                    | DiagStatus::cp0_condition(false),
            )
            | Status::interrupt_mask(0)
            | Status::kernel_addressing_64_bit(false)
            | Status::supervisor_addressing_64_bit(false)
            | Status::user_addressing_64_bit(false)
            | Status::mode(ExecutionMode::Kernel)
            | Status::error(false)
            | Status::exception(false)
            | Status::interrupt_enable(false),
    );

    cop0.disable_interrupts();

    let start = addr_of!(int_handler_ROM_START).addr();
    let end = addr_of!(int_handler_ROM_END).addr();
    let len = end - start;

    for entry in (0x80000000u32..=0x80000180).step_by(0x80) {
        let dst = from_raw_parts_mut::<Align8<[u8]>>(entry as *mut (), len)
            .as_mut()
            .expect("should never be null");

        let pi: &mut crate::pi::Pi = pi();
        pi.read_into(dst, start as _);

        instruction_cache_invalidate(&dst.0);
    }

    let im = im();
    // disable all interrupts except button
    im.update();

    cop0.enable_interrupts();

    //cop0.set_status(cop0.status() | 0x00000001);
}
