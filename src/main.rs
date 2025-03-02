#![no_std]
#![no_main]

#![allow(internal_features)]

#![feature(naked_functions)]
#![feature(ptr_as_ref_unchecked)]
#![feature(iter_array_chunks)]

extern crate alloc;

mod exception;
mod drivers;
mod memory;
mod cpu;

use core::arch::asm;
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
pub unsafe fn kmain() -> ! {
    memset(addr_of!(__bss) as *mut u8, 0, addr_of!(__bss_end) as usize - addr_of!(__bss) as usize);

    log!("initalizing kernel");

    exception::init();

    memory::init();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log!("kernel panic: {:?}", info);

    loop {}
}

#[no_mangle]
#[link_section = ".text.boot"]
#[naked]
pub unsafe extern "C" fn boot() {
    asm!(
        "la sp, __stack_top",
        "tail kmain",
        options(noreturn),
    );
}


