use core::ffi::c_void;
use core::panic::PanicInfo;
use core::ptr::{from_raw_parts, from_raw_parts_mut};
use core::{arch::asm, mem::size_of};

use crate::cop0::{DiagStatus, Status};
use crate::mi::mi;
use crate::pi::pi;
use crate::text::Colour;
use crate::vi::vi;
use crate::{mi::BB_SECURE_EXCEPTION, util::phys_to_k1_u32};

extern "Rust" {
    fn check_trial_timer();
    static SKC_TABLE: [fn() -> i32; 0];
    static SKC_TABLE_SIZE: usize;
}

#[link_section = ".boot"]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    pi().power_on();

    let vi = vi();

    //let mut buf = [0; 0x100];

    vi.init();
    vi.clear_framebuffer();

    let mut line = 1;

    vi.print_string(2, line, Colour::RED, "Panic");
    vi.wait_vsync();
    vi.next_framebuffer();

    loop {
        vi.clear_framebuffer();

        line = 1;

        vi.print_string(2, line, Colour::RED, "Panic");

        line += 1;
        if let Some(loc) = _info.location() {
            vi.print_string(2, line, Colour::WHITE, loc.file());
            line += 2;
            vi.print_u32(2, line, Colour::WHITE, loc.line());
            line += 1;
            vi.print_u32(2, line, Colour::WHITE, loc.column());
            line += 1;
        }
        if let Some(Some(msg)) = _info.message().map(|m| m.as_str()) {
            vi.print_string(2, line, Colour::WHITE, msg);
            line += 4;
        }
        if let Some(payload) = _info.payload().downcast_ref::<&'static str>() {
            vi.print_string(2, line, Colour::WHITE, payload);
            line += 4;
        }

        vi.wait_vsync();
        vi.next_framebuffer();
    }
    //loop {}
}

#[link_section = ".entry_uncached"]
#[no_mangle]
#[naked]
/// this is the first code that runs when coming from the bootrom, after soft reset, and various
/// other interrupt causes, and as such it can be entered either cached or uncached
///
/// the first step is to save kt0 and kt1, since we might have entered from a timer interrupt during
/// a libultra interrupt
/// needs some work to figure out how to handle libdragon interrupts
///
/// the shifts and add work to combine the 32-bit portions of kt0 and kt1 into kt1
///
/// we then jump to the cached function for doing actual stuff
pub unsafe extern "C" fn _startup() {
    asm!(
        ".set noreorder             ",
        "   dsll32  $kt0, $kt0, 0   ",
        "   dsrl32  $kt0, $kt0, 0   ",
        "   dsll32  $kt1, $kt1, 0   ",
        "   daddu   $kt1, $kt1, $kt0",
        "                           ",
        "   la      $kt0, {cached}  ",
        "   jr      $kt0            ",
        "    nop                    ",
        ".set reorder               ",
        cached = sym _startup_cached,
        options(noreturn),
    )
}

#[link_section = ".entry"]
#[naked]
unsafe extern "C" fn _startup_cached() {
    asm!(
        ".set noreorder             ",
        ".set noat                  ",
        "   lw      $kt0, {intreg}  ",
        "   and     $kt0, (1 << 2)  ",
        "   beqz    $kt0, 1f        ",
        "    nop                    ",
        "                           ",
        "   j {skc}                 ",
        "    move   $kt0, $sp       ",
        "                           ",
        "1:                         ",
        "   lw      $kt0, {intreg}  ",
        "   and     $kt0, (1 << 3)  ",
        "   beqzl   $kt0, 1f        ",
        "    nop                    ",
        "                           ",
        "   j {timer_expiry}        ",
        "    move   $kt0, $sp       ",
        "                           ",
        "1:                         ",
        "   lw      $kt0, {intreg}  ",
        "   and     $kt0, 0xF0      ",
        "   beqzl   $kt0, 1f        ",
        "    nop                    ",
        "                           ",
        "   j {other}               ",
        "    move   $kt0, $sp       ",
        "                           ",
        "1:                         ",
        "   j {startup}             ",
        "    nop                    ",
        ".set at                    ",
        ".set reorder               ",
        intreg = const phys_to_k1_u32(BB_SECURE_EXCEPTION) as i32,
        skc = sym handle_skc,
        timer_expiry = sym handle_timer_expiry,
        other = sym handle_other,
        startup = sym startup,
        options(noreturn),
    )
}

#[link_section = ".boot"]
#[naked]
pub unsafe extern "C" fn startup() {
    asm!(
        ".set noreorder             ",
        ".set noat                  ",
        "   la      $sp, {top}      ",
        "   la      $ra, {main}     ",
        "   j       {setup}         ",
        "    nop                    ",
        ".set at                    ",
        ".set reorder               ",
        top = sym super::__stack_end,
        setup = sym _setup,
        main = sym super::_main,
        options(noreturn)
    )
}

#[link_section = ".boot"]
unsafe extern "C" fn _setup() {
    super::clear_bss();
    /*from_raw_parts_mut::<u32>(phys_to_k1_u32(0x1FC0_07FC) as i32 as *mut (), ()).write_volatile(
        from_raw_parts::<u32>(phys_to_k1_u32(0x1FC0_07FC) as i32 as *mut (), ()).read_volatile()
            | 8,
    );*/
}

#[link_section = ".boot"]
#[naked]
unsafe extern "C" fn handle_skc() {
    const ABI_REQUIRED_SPACE: usize = /*4 * size_of::<u32>()*/ 0; // required in o32, but we're using n32
    const NUM_GPRS: usize = 12; // {s0..=s8} + {gp} + {sp} + {ra}
    const NUM_HILO: usize = 2;
    const NUM_GP32: usize = 0;
    const NUM_COP0: usize = 1; // count
    const EXCEPTION_STACK_DIFF: usize = (ABI_REQUIRED_SPACE
        + (size_of::<u64>() * NUM_GPRS)
        + (size_of::<u64>() * NUM_HILO)
        + (size_of::<u32>() * NUM_GP32)
        + (size_of::<u32>() * NUM_COP0))
        .next_multiple_of(size_of::<u128>()); // n32 requires stack aligned to 128 bits
    const GPRS_START: usize = ABI_REQUIRED_SPACE;
    const HILO_START: usize = GPRS_START + (size_of::<u64>() * NUM_GPRS);
    const GP32_START: usize = HILO_START + (size_of::<u64>() * NUM_HILO);
    const COP0_START: usize = GP32_START + (size_of::<u32>() * NUM_GP32);

    asm!(
        ".set noreorder                                     ",
        ".set noat                                          ",
        "   la      $sp, {top} - {diff}                     ",
        "                                                   ",
        "   sd      $16,  ({gprs} + 15 * 8)($sp)            ", // s0
        "   sd      $17,  ({gprs} + 16 * 8)($sp)            ", // s1
        "   sd      $18,  ({gprs} + 17 * 8)($sp)            ", // s2
        "   sd      $19,  ({gprs} + 18 * 8)($sp)            ", // s3
        "   sd      $20,  ({gprs} + 19 * 8)($sp)            ", // s4
        "   sd      $21,  ({gprs} + 20 * 8)($sp)            ", // s5
        "   sd      $22,  ({gprs} + 21 * 8)($sp)            ", // s6
        "   sd      $23,  ({gprs} + 22 * 8)($sp)            ", // s7
        "   sd      $28,  ({gprs} + 25 * 8)($sp)            ", // gp
        "   sd      $26,  ({gprs} + 26 * 8)($sp)            ", // sp - stored in kt0
        "   sd      $30,  ({gprs} + 27 * 8)($sp)            ", // s8
        "   sd      $31,  ({gprs} + 28 * 8)($sp)            ", // ra
        "                                                   ",
        "   mflo    $kt0                                    ",
        "   sd      $kt0, ({hilo} + 0 * 8)($sp)             ", // lo
        "   mfhi    $kt0                                    ",
        "   sd      $kt0, ({hilo} + 1 * 8)($sp)             ", // hi
        "                                                   ",
        "   mfc0    $kt0, $9                                ",
        "   sd      $kt0, ({cop0} + 0 * 4)($sp)             ", // count
        "                                                   ",
        ".set at                                            ",
        "                                                   ",
        "   lw      $s0, {skc_table_size}                   ",
        "   bgeu    $v0, $s0, 1f                            ", // may as well fix eSKape while we're at it
        "    nop                                            ",
        "                                                   ",
        "   sll     $s0, $v0, 2                             ",
        "   la      $s1, {skc_table}                        ",
        "   addu    $s2, $s1, $s0                           ",
        "   lw      $s3, 0($s2)                             ",
        "   jalr    $s3                                     ",
        "    nop                                            ",
        "                                                   ",
        "1:                                                 ",
        "   ld      $kt0, ({cop0} + 0 * 4)($sp)             ", // count
        "   mtc0    $kt0, $9                                ",
        "                                                   ",
        "   ld      $kt0, ({hilo} + 1 * 8)($sp)             ", // hi
        "   mthi    $kt0                                    ",
        "   ld      $kt0, ({hilo} + 0 * 8)($sp)             ", // lo
        "   mtlo    $kt0                                    ",
        "                                                   ",
        "   mfc0    $kt0, $12                               ", // sr
        "   and     $kt0, ~({bev_sr})                       ",
        "   mtc0    $kt0, $12                               ",
        "                                                   ",
        "   la      $kt1, {intreg}                          ",
        "   lw      $kt0, 0($kt1)                           ",
        "   and     $kt0, ~((1 << 6) | (1 << 2) | (1 << 0)) ", // don't write yet
        "                                                   ",
        ".set noat                                          ",
        "                                                   ",
        "   ld      $16,  ({gprs} + 15 * 8)($sp)            ", // s0
        "   ld      $17,  ({gprs} + 16 * 8)($sp)            ", // s1
        "   ld      $18,  ({gprs} + 17 * 8)($sp)            ", // s2
        "   ld      $19,  ({gprs} + 18 * 8)($sp)            ", // s3
        "   ld      $20,  ({gprs} + 19 * 8)($sp)            ", // s4
        "   ld      $21,  ({gprs} + 20 * 8)($sp)            ", // s5
        "   ld      $22,  ({gprs} + 21 * 8)($sp)            ", // s6
        "   ld      $23,  ({gprs} + 22 * 8)($sp)            ", // s7
        "   ld      $24,  ({gprs} + 23 * 8)($sp)            ", // t8
        "   ld      $25,  ({gprs} + 24 * 8)($sp)            ", // t9
        "   ld      $28,  ({gprs} + 25 * 8)($sp)            ", // gp
                                                               // sp is loaded last
        "   ld      $30,  ({gprs} + 27 * 8)($sp)            ", // s8
        "   ld      $31,  ({gprs} + 28 * 8)($sp)            ", // ra
        "                                                   ",
        "   ld      $29,  ({gprs} + 26 * 8)($sp)            ", // sp
        "                                                   ",
        ".balign 32                                         ",
        "   sw      $kt0, 0($kt1)                           ", // exit secure mode
        "   eret                                            ",
        "    nop                                            ",
        ".set at                                            ",
        ".set reorder                                       ",
        top = sym super::__stack_end,
        diff = const EXCEPTION_STACK_DIFF,
        gprs = const GPRS_START,
        hilo = const HILO_START,
        //gp32 = const GP32_START,
        cop0 = const COP0_START,
        skc_table_size = sym SKC_TABLE_SIZE,
        skc_table = sym SKC_TABLE,
        bev_sr = const (DiagStatus::bootstrap_exception_vectors(true) | DiagStatus::soft_reset(true)),
        intreg = const phys_to_k1_u32(BB_SECURE_EXCEPTION),
        options(noreturn)
    )
}

#[link_section = ".boot"]
unsafe extern "C" fn _check_trial_timer() {
    check_trial_timer();
}

#[link_section = ".boot"]
#[naked]
unsafe extern "C" fn handle_timer_expiry() {
    const ABI_REQUIRED_SPACE: usize = /*4 * size_of::<u32>()*/ 0; // required in o32, but we're using n32
    const NUM_GPRS: usize = 29; // 32 - {kt0, kt1} - {zero}
    const NUM_HILO: usize = 2;
    const NUM_GP32: usize = 2; // {kt0, kt1}
    const NUM_COP0: usize = 1; // count
    const EXCEPTION_STACK_DIFF: usize = (ABI_REQUIRED_SPACE
        + (size_of::<u64>() * NUM_GPRS)
        + (size_of::<u64>() * NUM_HILO)
        + (size_of::<u32>() * NUM_GP32) // we only have the bottom 32 bits of kt0 and kt1
        + (size_of::<u32>() * NUM_COP0))
        .next_multiple_of(size_of::<u128>()); // n32 requires stack aligned to 128 bits
    const GPRS_START: usize = ABI_REQUIRED_SPACE;
    const HILO_START: usize = GPRS_START + (size_of::<u64>() * NUM_GPRS);
    const GP32_START: usize = HILO_START + (size_of::<u64>() * NUM_HILO);
    const COP0_START: usize = GP32_START + (size_of::<u32>() * NUM_GP32);

    asm!(
        ".set noreorder                             ",
        ".set noat                                  ",
        "   la      $sp, {top} - {diff}             ",
        "                                           ",
        "   sd      $1,   ({gprs} +  0 * 8)($sp)    ", // at
        "   sd      $2,   ({gprs} +  1 * 8)($sp)    ", // v0
        "   sd      $3,   ({gprs} +  2 * 8)($sp)    ", // v1
        "   sd      $4,   ({gprs} +  3 * 8)($sp)    ", // a0
        "   sd      $5,   ({gprs} +  4 * 8)($sp)    ", // a1
        "   sd      $6,   ({gprs} +  5 * 8)($sp)    ", // a2
        "   sd      $7,   ({gprs} +  6 * 8)($sp)    ", // a3
        "   sd      $8,   ({gprs} +  7 * 8)($sp)    ", // a4
        "   sd      $9,   ({gprs} +  8 * 8)($sp)    ", // a5
        "   sd      $10,  ({gprs} +  9 * 8)($sp)    ", // a6
        "   sd      $11,  ({gprs} + 10 * 8)($sp)    ", // a7
        "   sd      $12,  ({gprs} + 11 * 8)($sp)    ", // t4
        "   sd      $13,  ({gprs} + 12 * 8)($sp)    ", // t5
        "   sd      $14,  ({gprs} + 13 * 8)($sp)    ", // t6
        "   sd      $15,  ({gprs} + 14 * 8)($sp)    ", // t7
        "   sd      $16,  ({gprs} + 15 * 8)($sp)    ", // s0
        "   sd      $17,  ({gprs} + 16 * 8)($sp)    ", // s1
        "   sd      $18,  ({gprs} + 17 * 8)($sp)    ", // s2
        "   sd      $19,  ({gprs} + 18 * 8)($sp)    ", // s3
        "   sd      $20,  ({gprs} + 19 * 8)($sp)    ", // s4
        "   sd      $21,  ({gprs} + 20 * 8)($sp)    ", // s5
        "   sd      $22,  ({gprs} + 21 * 8)($sp)    ", // s6
        "   sd      $23,  ({gprs} + 22 * 8)($sp)    ", // s7
        "   sd      $24,  ({gprs} + 23 * 8)($sp)    ", // t8
        "   sd      $25,  ({gprs} + 24 * 8)($sp)    ", // t9
        "   sd      $28,  ({gprs} + 25 * 8)($sp)    ", // gp
        "   sd      $26,  ({gprs} + 26 * 8)($sp)    ", // sp - stored in kt0
        "   sd      $30,  ({gprs} + 27 * 8)($sp)    ", // s8
        "   sd      $31,  ({gprs} + 28 * 8)($sp)    ", // ra
        "                                           ",
        "   mflo    $kt0                            ",
        "   sd      $kt0, ({hilo} + 0 * 8)($sp)     ", // lo
        "   mfhi    $kt0                            ",
        "   sd      $kt0, ({hilo} + 1 * 8)($sp)     ", // hi
        "                                           ",
        "   sw      $kt1, ({gp32} + 0 * 4)($sp)     ", // kt0
        "   dsrl32  $kt1, $kt1, 0                   ",
        "   sw      $kt1, ({gp32} + 1 * 4)($sp)     ", // kt1
        "                                           ",
        "   mfc0    $kt0, $9                        ",
        "   sd      $kt0, ({cop0} + 0 * 4)($sp)     ", // count
        "                                           ",
        ".set at                                    ",
        "                                           ",
        "   jal {check}                             ",
        "    nop                                    ",
        "                                           ",
        "   mfc0    $kt0, $12                       ", // sr
        "   and     $kt0, ~({bev_sr})               ",
        "   mtc0    $kt0, $12                       ",
        "                                           ",
        "   la      $kt1, {intreg}                  ",
        "   lw      $kt0, 0($kt1)                   ",
        "   and     $kt0, ~((1 << 3) | (1 << 0))    ", // don't write yet
        "                                           ",
        ".set noat                                  ",
        "                                           ",
        "   ld      $v0,  ({cop0} + 0 * 4)($sp)     ", // count
        "   mtc0    $v0, $9                         ",
        "                                           ",
        "   ld      $v0,  ({hilo} + 1 * 8)($sp)     ", // hi
        "   mthi    $v0                             ",
        "   ld      $v0,  ({hilo} + 0 * 8)($sp)     ", // lo
        "   mtlo    $v0                             ",
        "                                           ",
        "   ld      $1,   ({gprs} +  0 * 8)($sp)    ", // at
        "   ld      $2,   ({gprs} +  1 * 8)($sp)    ", // v0
        "   ld      $3,   ({gprs} +  2 * 8)($sp)    ", // v1
        "   ld      $4,   ({gprs} +  3 * 8)($sp)    ", // a0
        "   ld      $5,   ({gprs} +  4 * 8)($sp)    ", // a1
        "   ld      $6,   ({gprs} +  5 * 8)($sp)    ", // a2
        "   ld      $7,   ({gprs} +  6 * 8)($sp)    ", // a3
        "   ld      $8,   ({gprs} +  7 * 8)($sp)    ", // a4
        "   ld      $9,   ({gprs} +  8 * 8)($sp)    ", // a5
        "   ld      $10,  ({gprs} +  9 * 8)($sp)    ", // a6
        "   ld      $11,  ({gprs} + 10 * 8)($sp)    ", // a7
        "   ld      $12,  ({gprs} + 11 * 8)($sp)    ", // t4
        "   ld      $13,  ({gprs} + 12 * 8)($sp)    ", // t5
        "   ld      $14,  ({gprs} + 13 * 8)($sp)    ", // t6
        "   ld      $15,  ({gprs} + 14 * 8)($sp)    ", // t7
        "   ld      $16,  ({gprs} + 15 * 8)($sp)    ", // s0
        "   ld      $17,  ({gprs} + 16 * 8)($sp)    ", // s1
        "   ld      $18,  ({gprs} + 17 * 8)($sp)    ", // s2
        "   ld      $19,  ({gprs} + 18 * 8)($sp)    ", // s3
        "   ld      $20,  ({gprs} + 19 * 8)($sp)    ", // s4
        "   ld      $21,  ({gprs} + 20 * 8)($sp)    ", // s5
        "   ld      $22,  ({gprs} + 21 * 8)($sp)    ", // s6
        "   ld      $23,  ({gprs} + 22 * 8)($sp)    ", // s7
        "   ld      $24,  ({gprs} + 23 * 8)($sp)    ", // t8
        "   ld      $25,  ({gprs} + 24 * 8)($sp)    ", // t9
        "   ld      $28,  ({gprs} + 25 * 8)($sp)    ", // gp
                                                       // sp is loaded last
        "   ld      $30,  ({gprs} + 27 * 8)($sp)    ", // s8
        "   ld      $31,  ({gprs} + 28 * 8)($sp)    ", // ra
        "                                           ",
        ".balign 32                                 ",
        "   sw      $kt0, 0($kt1)                   ", // exit secure mode
        "   lw      $kt1, ({gp32} +  1 * 4)($sp)    ", // kt1
        "   lw      $kt0, ({gp32} +  0 * 4)($sp)    ", // kt0
        "   ld      $29,  ({gprs} + 26 * 8)($sp)    ", // sp
        "   eret                                    ",
        "    nop                                    ",
        ".set at                                    ",
        ".set reorder                               ",
        top = sym super::__stack_end,
        diff = const EXCEPTION_STACK_DIFF,
        gprs = const GPRS_START,
        hilo = const HILO_START,
        gp32 = const GP32_START,
        cop0 = const COP0_START,
        check = sym _check_trial_timer,
        bev_sr = const (DiagStatus::bootstrap_exception_vectors(true) | DiagStatus::soft_reset(true)),
        intreg = const phys_to_k1_u32(BB_SECURE_EXCEPTION),
        options(noreturn)
    )
}

#[link_section = ".boot"]
unsafe extern "C" fn handle_other() {
    if mi().bb_secure_exception() & (1 << 6) != 0 {
        // button
        startup();
    }
    panic!("unhandled entry type");
}

#[link_section = ".int_handler"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _int_handler() {
    asm!(
        ".set noreorder             ",
        "   la      $kt0, {cached}  ",
        "   jr      $kt0            ",
        "    nop                    ",
        ".set reorder               ",
        cached = sym _int_handler_cached,
        options(noreturn),
    )
}

#[link_section = ".int_handler"]
unsafe extern "C" fn _int_handler_cached() {
    panic!("interrupt");
}

#[naked]
pub unsafe extern "C" fn launch_app(entry: unsafe extern "C" fn(u32) -> !) -> ! {
    asm!(
        ".set noreorder                 ",
        "   move    $v0, $a0            ",
        "                               ",
        "   mfc0    $a0, $12            ", // sr
        "   and     $a0, ~({bev_sr_ie}) ",
        "   mtc0    $a0, $12            ",
        "                               ",
        "   la      $t1, {intreg}       ",
        "   lw      $a0, 0($t1)         ",
        "   and     $t0, $a0, ~0xFD     ",
        "                               ",
        ".balign 32                     ",
        "   sw      $t0, 0($t1)         ",
        "   jr      $v0                 ",
        "    nop                        ",
        ".set reorder                   ",
        bev_sr_ie = const (DiagStatus::bootstrap_exception_vectors(true) | DiagStatus::soft_reset(true) | Status::error(true) | Status::exception(true) | Status::interrupt_enable(true)),
        intreg = const phys_to_k1_u32(BB_SECURE_EXCEPTION),
        options(noreturn)
    )
}
