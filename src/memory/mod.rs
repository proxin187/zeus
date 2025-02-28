use crate::log;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::addr_of;

const CHUNK_LIMIT: usize = 999;


#[global_allocator]
static mut ALLOC: Allocator = Allocator::new();

extern "C" {
    static mut __ram: u8;
    static mut __ram_end: u8;
}

#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    base: u64,
    length: u64,
}

impl Chunk {
    pub fn new(base: u64, length: u64) -> Chunk {
        Chunk {
            base,
            length,
        }
    }
}

pub struct Allocator {
    chunks: [Option<Chunk>; CHUNK_LIMIT],
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            chunks: [None; CHUNK_LIMIT],
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!();
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        todo!();
    }
}

pub fn init() {
    unsafe {
        ALLOC.chunks[0].replace(Chunk::new(addr_of!(__ram) as u64, addr_of!(__ram_end) as u64 - addr_of!(__ram) as u64));

        log!("allocator initialized chunks={:#x?}", ALLOC.chunks);
    }
}


