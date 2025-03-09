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
mod memory;
mod cpu;

use core::arch::naked_asm;
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
pub unsafe extern "C" fn proc1() {
    /*
    naked_asm!(
        "add sp, sp, 20",
        "add a0, a0, 1",
        "add sp, sp, -20",

        "j proc1",
    );
    */

    loop {
        log!("proc1 test");

        for _ in 0..10000000 {}
    }
}

#[no_mangle]
pub unsafe fn kmain() -> ! {
    memset(addr_of!(__bss) as *mut u8, 0, addr_of!(__bss_end) as usize - addr_of!(__bss) as usize);

    log!("initalizing kernel");

    exception::init();

    memory::init();

    process::spawn("init", proc1 as u64);

    exception::init_timer();

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


