#![no_std]
#![no_main]

#![feature(naked_functions)]

mod drivers;

use core::arch::asm;
use core::panic::PanicInfo;


pub unsafe fn memset(buf: *mut u8, value: u8, count: usize) {
    for offset in 0..count {
        *buf.add(offset) = value;
    }
}

#[no_mangle]
pub unsafe fn kmain() -> ! {
    // memset(__bss, 0, __bss_end as usize - __bss as usize);

    drivers::init();

    loop {}
}

#[panic_handler]
fn _panic(_info: &PanicInfo) -> ! {
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


