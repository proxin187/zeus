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

    pub fn read(&mut self) -> Option<u8> {
        unsafe {
            (self.uart.add(5).read_volatile() & 1 != 0)
                .then(|| self.uart.read_volatile())
        }
    }

    #[inline]
    pub fn write(&mut self, byte: u8) {
        unsafe {
            UART.uart.write_volatile(byte);
        }
    }
}

impl fmt::Write for Uart {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for byte in string.bytes() {
            self.write(byte);
        }

        Ok(())
    }
}


