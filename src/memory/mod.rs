use crate::log;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{self, addr_of};
use core::mem;
use core::slice;

const CHUNK_LIMIT: usize = 999;


#[global_allocator]
pub static mut ALLOC: Allocator = Allocator::new();

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
    pub const fn new(base: u64, length: u64) -> Chunk {
        Chunk {
            base,
            length,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.base == 0 && self.length == 0
    }
}

pub struct Allocator {
    chunks: *mut [Chunk; CHUNK_LIMIT],
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            chunks: ptr::null_mut(),
        }
    }

    pub unsafe fn merge(&self) {
        slice::sort::quicksort(self.chunks.as_mut_unchecked(), |a, b| a.base > b.base);

        for [a, b] in (*self.chunks).iter().enumerate().flat_map(|(a, _)| [a, a + 1]).array_chunks() {
            if (*self.chunks)[a].base + (*self.chunks)[a].length == (*self.chunks)[b].base {
                (*self.chunks)[a].length += (*self.chunks)[b].length;

                (*self.chunks)[b] = Chunk::new(0, 0);
            }
        }
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match (*self.chunks).iter_mut().filter(|chunk| chunk.length > layout.size() as u64).next() {
            Some(chunk) => {
                chunk.length -= layout.size() as u64;

                (chunk.base + chunk.length) as *mut u8
            },
            None => {
                panic!("unable to allocate: out of memory");
            },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match (*self.chunks).iter_mut().filter(|chunk| chunk.is_empty()).next() {
            Some(chunk) => {
                *chunk = Chunk::new(ptr as u64, layout.size() as u64);
            },
            None => {
                panic!("unable to deallocate: out of chunks");
            },
        }

        self.merge();
    }
}

pub fn init() {
    unsafe {
        ALLOC.chunks = addr_of!(__ram) as *mut [Chunk; CHUNK_LIMIT];

        let base = addr_of!(__ram) as u64 + mem::size_of::<[Chunk; CHUNK_LIMIT]>() as u64;

        (*ALLOC.chunks)[0] = Chunk::new(base, addr_of!(__ram_end) as u64 - base);

        log!("allocator initialized chunk table addr={:#x?}, limit={}", ALLOC.chunks, CHUNK_LIMIT);
    }
}


