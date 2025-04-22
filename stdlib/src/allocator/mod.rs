use crate::sys;

use core::alloc::{GlobalAlloc, Layout};


#[global_allocator]
pub static mut ALLOC: MemoryAllocator = MemoryAllocator::new();

pub struct MemoryAllocator;

impl MemoryAllocator {
    pub const fn new() -> MemoryAllocator { MemoryAllocator }
}

unsafe impl GlobalAlloc for MemoryAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            sys::alloc(layout.size(), layout.align())
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            sys::dealloc(layout.size(), layout.align(), ptr)
        }
    }
}



