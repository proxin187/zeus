#![no_std]
#![no_main]

#![feature(naked_functions)]

use core::arch::asm;
use core::panic::PanicInfo;


extern "C" {
    pub static __bss: *mut u8;
    pub static __bss_end: *mut u8;
    pub static __stack_top: *mut u8;
}

pub unsafe fn memset(buf: *mut u8, value: u8, count: usize) {
    for offset in 0..count {
        (*buf.add(offset)) = value;
    }
}

#[no_mangle]
pub unsafe fn kmain() -> ! {
    memset(__bss, 0, __bss_end as usize - __bss as usize);

    loop {}
}

#[panic_handler]
fn _panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
#[link_section = ".text.boot"]
#[naked]
pub unsafe extern "C" fn bootloader() {
    asm!(
        "la sp, __stack_top",
        "j kmain",
        options(noreturn),
    );
}


