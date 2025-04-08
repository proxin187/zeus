use crate::drivers::uart;
use crate::log;

use core::arch::asm;
use core::iter;

use alloc::string::String;


pub struct Shell {
    cwd: [u8; 56],
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            cwd: [0; 56],
        }
    }

    fn command(&self) -> String {
        let bytes = iter::repeat_with(|| unsafe { uart::UART.read() })
            .filter_map(|byte| byte.map(|byte| byte as char))
            .take_while(|read| *read != '\n');

        bytes.collect()
    }

    pub fn run(&mut self) -> ! {
        log!("welcome to dnb shell");

        loop {
            let command = self.command();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn entry() -> ! {
    let mut shell = Shell::new();

    shell.run()
}


