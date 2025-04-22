#![no_std]

#![cfg(feature = "allocator")]
pub mod allocator;

pub mod process;
pub mod error;
pub mod sys;
pub mod fs;
pub mod io;


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


