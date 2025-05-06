use core::arch::naked_asm;
use core::array;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;
use core::ptr::{
    addr_of, addr_of_mut, from_raw_parts, from_raw_parts_mut, with_exposed_provenance_mut,
};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::ToString;

pub mod globals;
pub mod interrupts;

use super::is_bbplayer;
use globals::{osTvType, setup_globals};
use interrupts::setup_ints;

use crate::si::si;
use crate::text::Colour;
use crate::util::phys_to_k1_usize;
use crate::vi::{vi, Mode};

const IPL3_SIZE: usize = 0x1000 - 0x40;

fn to_base_10<const N: usize>(num: u32) -> ([u8; N], usize) {
    let arr =
        array::from_fn::<_, N, _>(|i| ((num / (10u32.pow((N - i - 1) as _))) % 10) as u8 + b'0');
    let index = arr
        .iter()
        .enumerate()
        .find_map(|(i, &e)| if e != 0 { Some(i) } else { None })
        .unwrap_or(N - 1);
    (arr, index)
}

#[link_section = ".boot"]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let si = si();

    si.init_hw();
    si.txrx(b"Panic\n\n", None);
    if let Some(loc) = _info.location() {
        si.txrx(loc.file().as_bytes(), None);
        si.txrx(b":", None);
        let line = loc.line();
        let (line_buf, line_start) = to_base_10::<10>(line);
        si.txrx(&line_buf[line_start..], None);
        si.txrx(b":", None);
        let column = loc.column();
        let (column_buf, column_start) = to_base_10::<10>(column);
        si.txrx(&column_buf[column_start..], None);
        si.txrx(b"\n\n", None);
    }
    if let Some(msg) = _info.message().as_str() {
        si.txrx(msg.as_bytes(), None);
        si.txrx(b"\n\n", None);
    }

    let vi = vi();

    //let mut buf = [0; 0x100];

    #[allow(static_mut_refs)]
    vi.init(match unsafe { (&raw const osTvType).read_volatile() } {
        0 => Mode::PAL60,
        1 => Mode::NTSC,
        2 => todo!(),
        _ => unreachable!(),
    });
    vi.clear_framebuffer();

    let mut line = 1;

    vi.print_string(2, line, Colour::RED, "Panic");
    vi.wait_vsync();
    vi.next_framebuffer();

    loop {
        vi.clear_framebuffer();

        line = 1;

        vi.print_string(2, line, Colour::RED, "Panic");

        #[cfg(feature = "alloc")]
        {
            line += 1;
            if let Some(loc) = _info.location() {
                vi.print_string(2, line, Colour::WHITE, &loc.to_string());
                line += 4;
            }

            vi.print_string(2, line, Colour::WHITE, &_info.message().to_string());
            line += 4;
        }
        #[cfg(not(feature = "alloc"))]
        {
            line += 1;
            if let Some(loc) = _info.location() {
                vi.print_string(2, line, Colour::WHITE, loc.file());
                line += 2;
                vi.print_u32(2, line, Colour::WHITE, loc.line());
                line += 1;
                vi.print_u32(2, line, Colour::WHITE, loc.column());
                line += 1;
            }
            if let Some(msg) = _info.message().as_str() {
                vi.print_string(2, line, Colour::WHITE, msg);
                line += 4;
            }
            if let Some(payload) = _info.payload().downcast_ref::<&'static str>() {
                vi.print_string(2, line, Colour::WHITE, payload);
                line += 4;
            }
        }

        vi.wait_vsync();
        vi.next_framebuffer();
    }
    //loop {}
}

#[cfg(not(feature = "ipl3"))]
#[link_section = ".ipl3"]
#[used]
static mut IPL3: MaybeUninit<[u8; IPL3_SIZE]> = MaybeUninit::uninit();

#[cfg(feature = "ipl3")]
#[link_section = ".ipl3"]
#[used]
static mut IPL3: [u8; IPL3_SIZE] = include!(concat!(env!("OUT_DIR"), "/ipl3.rs"));

#[link_section = ".boot"]
unsafe extern "C" fn _setup() {
    super::clear_bss();
    #[cfg(feature = "alloc")]
    super::ALLOCATOR.init(
        addr_of_mut!(super::__heap_start),
        addr_of!(super::__heap_len).addr(),
    );
    setup_globals();
    if !is_bbplayer() {
        si().wait();
        with_exposed_provenance_mut::<u32>(phys_to_k1_usize(0x1FC0_07FC)).write_volatile(8);
    }
    setup_ints();
}

#[link_section = ".entry"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _startup() {
    naked_asm!(
        ".set noreorder         ",
        "  jal  {setup}         ",
       r"   lui $sp, %hi({top}) ",
        "  j    {main}          ",
        "   nop                 ",
        ".set reorder           ",
        setup = sym _setup,
        top = sym super::__stack_end,
        main = sym super::_main,
    )
}
