
#[link_section = ".rodata"]
pub static mut UART: Uart = Uart::new();


pub struct Uart {
    uart: *mut u8,
}

impl Uart {
    pub const fn new() -> Uart {
        Uart {
            uart: 0x10000000 as *mut u8,
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            unsafe {
                *self.uart = *byte;
            }
        }
    }
}


