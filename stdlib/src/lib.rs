#![no_std]

pub mod process;
pub mod error;
pub mod sys;
pub mod fs;
pub mod io;

#[cfg(feature = "userspace")]
pub mod allocator;

use core::panic::PanicInfo;


#[cfg(feature = "userspace")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = io::_print(format_args!("{}", info));

    loop {}
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        let _ = stdlib::io::_print(format_args!($($arg)*));

        let _ = stdlib::io::_print(format_args!("\n"));
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = stdlib::io::_print(format_args!($($arg)*));
    };
}


