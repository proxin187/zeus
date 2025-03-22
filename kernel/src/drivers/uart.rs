use core::fmt;


#[link_section = ".rodata"]
pub static mut UART: Uart = Uart::new();


pub struct Uart {
    uart: *mut u8,
}

impl Uart {
    const fn new() -> Uart {
        Uart {
            uart: 0x10000000 as *mut u8,
        }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for byte in string.bytes() {
            unsafe {
                *UART.uart = byte;
            }
        }

        Ok(())
    }
}


