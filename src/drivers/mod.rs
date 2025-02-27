mod uart;

use core::fmt;

#[allow(static_mut_refs)]


pub fn _print(args: fmt::Arguments) -> fmt::Result {
    unsafe {
        fmt::write(&mut uart::UART, args)
    }
}

pub fn _println(args: fmt::Arguments) -> fmt::Result {
    _print(args)?;

    _print(format_args!("\n"))
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        let _ = crate::drivers::_print(format_args!("[info] "));

        let _ = crate::drivers::_println(format_args!($($arg)*));
    };
}


