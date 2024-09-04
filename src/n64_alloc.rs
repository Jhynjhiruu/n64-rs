use core::alloc::GlobalAlloc;
use core::cell::RefCell;
use core::ffi::c_void;

use good_memory_allocator::Allocator;

pub struct N64Allocator(RefCell<Allocator>);

impl N64Allocator {
    const fn empty() -> Self {
        Self(RefCell::new(Allocator::empty()))
    }

    pub unsafe fn init(&self, heap_start_addr: *mut c_void, heap_size: usize) {
        let mut borrowed = self.0.borrow_mut();
        borrowed.init(heap_start_addr.addr(), heap_size)
    }
}

unsafe impl Sync for N64Allocator {}

unsafe impl GlobalAlloc for N64Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut borrowed = self.0.borrow_mut();
        borrowed.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut borrowed = self.0.borrow_mut();
        borrowed.dealloc(ptr)
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        let mut borrowed = self.0.borrow_mut();
        borrowed.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
pub static ALLOCATOR: N64Allocator = N64Allocator::empty();
