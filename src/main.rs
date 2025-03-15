#![no_std]
#![no_main]

#![allow(static_mut_refs)]
#![allow(internal_features)]

#![feature(naked_functions)]
#![feature(ptr_as_ref_unchecked)]
#![feature(iter_array_chunks)]

extern crate alloc;

mod exception;
mod drivers;
mod process;
mod syscall;
mod memory;
mod cpu;
mod fs;

use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;
use core::ptr::addr_of;

extern "C" {
    static mut __bss: u8;
    static mut __bss_end: u8;
}

pub unsafe fn memset(buf: *mut u8, value: u8, count: usize) {
    for offset in 0..count {
        *buf.add(offset) = value;
    }
}

#[no_mangle]
pub unsafe extern "C" fn init_proc() {
    loop {
        log!("test");

        for _ in 0..1000000 {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn proc2() {
    log!("exiting proc2");

    asm!(
        "li a0, 1",
        "li a7, 93",
        "ecall",
    );

    loop {}
}

#[no_mangle]
pub unsafe fn kmain() -> ! {
    memset(addr_of!(__bss) as *mut u8, 0, addr_of!(__bss_end) as usize - addr_of!(__bss) as usize);

    log!("initalizing kernel");

    exception::init();

    memory::init();

    use virtio_drivers::device::blk::VirtIOBlk;
    use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
    use core::ptr::NonNull;

    // TODO: compare with example to see what we do wrong
    let header = NonNull::new(0x10001000 as *mut VirtIOHeader).unwrap();

    log!("transport");

    let transport = MmioTransport::new(header, 0x1000).unwrap();

    log!("blk");

    let mut blk = VirtIOBlk::<drivers::virtio::HalImpl, _>::new(transport).unwrap();

    let mut output = alloc::vec![0; 512];

    log!("read");

    blk.read_blocks(0, &mut output).unwrap();

    let string = alloc::string::String::from_utf8(output);

    log!("string: {:?}", string);

    process::spawn("init", init_proc as u64);
    process::spawn("proc2", proc2 as u64);

    exception::enter_user();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log!("kernel panic: {:#?}", info);

    loop {}
}

#[no_mangle]
#[link_section = ".text.boot"]
#[naked]
pub unsafe extern "C" fn boot() {
    naked_asm!(
        "la sp, __stack_top",
        "call kmain",
    );
}


