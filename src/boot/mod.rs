use core::ffi::{c_int, c_void};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr::{addr_of, addr_of_mut};

#[cfg(not(feature = "sk"))]
mod n64;
#[cfg(feature = "sk")]
mod sk;

#[cfg(not(feature = "sk"))]
pub use n64::{globals, interrupts};
#[cfg(feature = "sk")]
pub use sk::launch_app;

//use crate::util::show;

#[cfg(feature = "alloc")]
use crate::n64_alloc::ALLOCATOR;
use crate::{data_cache_invalidate, data_cache_writeback_raw};

extern "Rust" {
    fn main() -> !;
}

extern "C" {
    static mut __bss_start: c_void;
    static __bss_size: c_void;

    static mut __heap_start: c_void;
    static __heap_len: c_void;

    static mut __stack_end: c_void;
}

const STACK_SIZE: usize = 0x4000;

#[link_section = ".stack"]
#[used]
static mut BOOTSTACK: MaybeUninit<[u8; STACK_SIZE]> = MaybeUninit::uninit();

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
unsafe fn clear_bss() {
    let start = addr_of_mut!(__bss_start).cast::<u8>();
    let size = addr_of!(__bss_size).addr();
    start.write_bytes(0, size);
    data_cache_writeback_raw(start.addr(), start.addr() + size);
}

pub fn is_bbplayer() -> bool {
    #[cfg(not(feature = "sk"))]
    unsafe {
        (&raw const globals::__osBbIsBb).read_volatile() != 0
    }
    #[cfg(feature = "sk")]
    true
}

pub fn cpu_speed() -> u32 {
    if is_bbplayer() {
        144000000
    } else {
        93750000
    }
}

pub fn ms_to_ticks(num: u32) -> u32 {
    μs_to_ticks(num * 1_000)
}

pub fn μs_to_ticks(num: u32) -> u32 {
    (((num as u64) * (cpu_speed() as u64)) / 1_000_000) as u32
}

pub fn rcp_speed() -> u32 {
    if is_bbplayer() {
        96000000
    } else {
        62500000
    }
}
