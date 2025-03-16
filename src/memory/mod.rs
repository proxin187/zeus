use crate::log;

use core::alloc::{GlobalAlloc, Allocator, Layout, AllocError};
use core::ptr::{self, NonNull, addr_of};
use core::mem;

use alloc::alloc;

const CHUNK_LIMIT: usize = 999;


#[global_allocator]
pub static mut ALLOC: MemoryAllocator = MemoryAllocator::new();

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

pub struct MemoryAllocator {
    chunks: *mut [Chunk; CHUNK_LIMIT],
}

impl MemoryAllocator {
    pub const fn new() -> MemoryAllocator {
        MemoryAllocator {
            chunks: ptr::null_mut(),
        }
    }

    pub unsafe fn push(&self, new: Chunk) {
        match (*self.chunks).iter_mut().filter(|chunk| chunk.is_empty()).next() {
            Some(chunk) => {
                *chunk = new;
            },
            None => {
                panic!("unable to push: out of chunks");
            },
        }
    }

    pub unsafe fn merge(&self) -> bool {
        let mut merged = false;

        for (a, _) in (*self.chunks).iter().enumerate().filter(|(_, chunk)| !chunk.is_empty()) {
            for (b, _) in (*self.chunks).iter().enumerate().filter(|(_, chunk)| !chunk.is_empty()) {
                if (*self.chunks)[a].base + (*self.chunks)[a].length == (*self.chunks)[b].base {
                    (*self.chunks)[a].length += (*self.chunks)[b].length;

                    (*self.chunks)[b] = Chunk::new(0, 0);

                    merged = true;
                }
            }
        }

        merged
    }

    pub unsafe fn defrag(&self) {
        while unsafe { self.merge() } {}
    }

    pub unsafe fn align(&self, chunk: &Chunk, align: u64, size: u64) {
        if chunk.length - (chunk.length & !(align - 1)) > 0 {
            self.push(Chunk::new(chunk.base as u64 + (chunk.length & !(align - 1)) + size, chunk.length - (chunk.length & !(align - 1))));
        }
    }
}

unsafe impl GlobalAlloc for MemoryAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match (*self.chunks).iter_mut().filter(|chunk| chunk.length > layout.size() as u64 + layout.align() as u64).next() {
            Some(chunk) => {
                chunk.length -= layout.size() as u64;

                self.align(&chunk, layout.align() as u64, layout.size() as u64);

                chunk.length &= !(layout.align() as u64 - 1);

                (chunk.base + chunk.length) as *mut u8
            },
            None => {
                panic!("unable to allocate: out of memory");
            },
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.push(Chunk::new(ptr as u64, layout.size() as u64));

        self.defrag();
    }
}

pub struct AlignedAlloc<const N: usize>;

unsafe impl<const N: usize> Allocator for AlignedAlloc<N> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        alloc::Global.allocate(layout.align_to(N).unwrap())
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        alloc::Global.deallocate(ptr, layout.align_to(N).unwrap())
    }
}

pub fn dump() {
    log!("dump chunks:");

    unsafe {
        let mut chunks = (*ALLOC.chunks).iter().filter(|chunk| !chunk.is_empty());

        while let Some(chunk) = chunks.next() {
            log!("chunk: {:?}", chunk);
        }
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


