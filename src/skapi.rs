use core::arch::global_asm;
use core::ffi::{c_int, c_void};
use core::mem::MaybeUninit;

use crate::mi::BB_SECURE_EXCEPTION;
use crate::util::phys_to_k1_u32;

type Result<T> = core::result::Result<T, ()>;

macro_rules! skc_call {
    ($n:expr, $e:expr) => {
        global_asm!(
            ".set noreorder     ",
            ".globl {name}      ",
            "{name}:            ",
            "  li  $v0, {id}    ",
            "  lw  $zero, {trap}",
            "  nop              ",
            "  nop              ",
            "  jr  $ra          ",
            "   nop             ",
            ".set reorder       ",
            name = sym $n,
            id = const $e,
            trap = const phys_to_k1_u32(BB_SECURE_EXCEPTION) as i32 // needed because of LLVM shenanigans
        );
    };
}

extern "C" {
    fn _get_id(id: *mut u32) -> c_int;
    fn _launch_setup(bundle: *mut c_void, crls: *mut c_void, recrypt_list: *mut c_void) -> c_int;
    fn _launch(address: *const c_void) -> c_int;
    fn _exit() -> !;
    fn _keep_alive();
}

skc_call!(_get_id, 0);
skc_call!(_launch_setup, 1);
skc_call!(_launch, 2);
skc_call!(_exit, 13);
skc_call!(_keep_alive, 14);

pub fn get_id() -> Result<u32> {
    let mut id = MaybeUninit::uninit();
    let status = unsafe { _get_id(id.as_mut_ptr()) };
    if status < 0 {
        Err(())
    } else {
        Ok(unsafe { id.assume_init() })
    }
}

pub fn launch_setup(
    bundle: *mut c_void,
    crls: *mut c_void,
    recrypt_list: *mut c_void,
) -> Result<()> {
    let status = unsafe { _launch_setup(bundle, crls, recrypt_list) };
    if status < 0 {
        Err(())
    } else {
        Ok(())
    }
}

pub fn launch(address: *const ()) -> Result<()> {
    let status = unsafe { _launch(address.cast()) };
    if status < 0 {
        Err(())
    } else {
        Ok(())
    }
}

pub fn exit() -> ! {
    unsafe { _exit() }
}

pub fn keep_alive() {
    unsafe { _keep_alive() }
}
