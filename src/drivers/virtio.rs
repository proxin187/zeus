use crate::log;

use core::sync::atomic::{Ordering, AtomicUsize};
use core::ptr::{addr_of, NonNull};

use virtio_drivers::{BufferDirection, Hal, PhysAddr, PAGE_SIZE};
use spin::Lazy;

extern "C" {
    static mut __virtq: u8;
}

static ADDR: Lazy<AtomicUsize> = Lazy::new(|| AtomicUsize::new(addr_of!(__virtq) as usize));

pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let addr = ADDR.fetch_add(PAGE_SIZE * pages, Ordering::SeqCst);

        log!("allocating virtio: {:#x?}", addr);

        (addr, NonNull::new(addr as _).expect("failed to create non-null"))
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        // memory::ALLOC.dealloc(paddr as *mut u8, Layout::from_size_align_unchecked(PAGE_SIZE * pages, PAGE_SIZE));

        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as *mut _).expect("failed to create non-null")
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        buffer.as_ptr() as *mut u8 as usize
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}

