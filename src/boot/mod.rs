use core::arch::asm;
use core::ffi::{c_int, c_void};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;
use core::ptr::{addr_of, addr_of_mut, from_raw_parts, from_raw_parts_mut};

pub mod globals;
mod interrupts;

use globals::{is_bbplayer, setup_globals};
use interrupts::setup_ints;

use crate::text::Colour;
use crate::util::phys_to_k1_usize;
//use crate::util::show;
use crate::vi::vi;
//use crate::n64_alloc::ALLOCATOR;

extern "Rust" {
    fn main() -> !;
}

#[link_section = ".boot"]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let vi = vi();

    //let mut buf = [0; 0x100];

    vi.init();

    loop {
        vi.clear_framebuffer();

        //let mut line = 1;

        vi.print_string(2, /* line */ 1, Colour::RED, "Panic");
        /*line += 1;
        if let Some(loc) = info.location() {
            let s = show(&mut buf, format_args!("{}:{}:{}", loc.file(), loc.line(), loc.column())).unwrap();
            vi.print_string(2, line, Colour::WHITE, s);
            line += 1;
        }

        if let Some(msg) = info.message() {
            let s = show(&mut buf, msg.clone()).unwrap();
            vi.print_string(2, line, Colour::WHITE, s);
            line += 1;
        }*/

        vi.wait_vsync();
        vi.next_framebuffer();
    }
    //loop {}
}

extern "C" {
    static mut __bss_start: c_void;
    static __bss_size: c_void;

    static mut _heap_start: c_void;
    static _heap_len: c_void;

    static mut __stack_end: c_void;
}

const STACK_SIZE: usize = 0x4000;
const IPL3_SIZE: usize = 0x1000 - 0x40;

#[link_section = ".stack"]
#[used]
static mut BOOTSTACK: MaybeUninit<[u8; STACK_SIZE]> = MaybeUninit::uninit();

#[cfg(not(feature = "ipl3"))]
#[link_section = ".ipl3"]
#[used]
static mut IPL3: MaybeUninit<[u8; IPL3_SIZE]> = MaybeUninit::uninit();

#[cfg(feature = "ipl3")]
#[link_section = ".ipl3"]
#[used]
static mut IPL3: [u8; IPL3_SIZE] = include!(concat!(env!("OUT_DIR"), "/ipl3.rs"));

#[link_section = ".boot"]
unsafe extern "C" fn clear_bss() {
    let start = addr_of_mut!(__bss_start).cast::<u8>();
    let size = addr_of!(__bss_size).addr();
    start.write_bytes(0, size);
}

/*#[link_section = ".entry"]
#[no_mangle]
pub unsafe fn start() -> ! {
    clear_bss();
    //ALLOCATOR.init(addr_of_mut!(_heap_start), addr_of!(_heap_len).addr());
    main();
}*/

#[link_section = ".boot"]
unsafe extern "C" fn _main() -> ! {
    main()
}

#[link_section = ".boot"]
unsafe extern "C" fn _setup() {
    clear_bss();
    //ALLOCATOR.init(addr_of_mut!(_heap_start), addr_of!(_heap_len).addr());
    setup_globals();
    if !is_bbplayer() {
        from_raw_parts_mut::<u32>(phys_to_k1_usize(0x1FC0_07FC) as _, ()).write_volatile(8);
    }
    setup_ints();
}

#[link_section = ".entry"]
#[no_mangle]
#[naked]
pub unsafe extern "C" fn _startup() {
    asm!(
         ".set noreorder        ",
         "  jal  {setup}        ",
        r"   lui $sp, %hi({top})",
         "  j    {main}         ",
         "   nop                ",
         ".set reorder          ",
        setup = sym _setup,
        top = sym __stack_end,
        main = sym _main,
        options(noreturn),
    )
}
