use crate::drivers::uart;
use crate::log;

use alloc::string::String;
use alloc::vec::Vec;

use core::fmt::Write;


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
        // TODO: decide wether we port the rust std or make our own
        let mut bytes: Vec<char> = Vec::new();

        unsafe {
            let _ = write!(uart::UART, "\n[shell]$ ");
        }

        while !bytes.ends_with(&['\n']) && !bytes.ends_with(&['\r']) {
            if let Some(byte) = unsafe { uart::UART.read() } {
                unsafe {
                    uart::UART.write(byte);
                }

                bytes.push(byte as char);
            }
        }

        bytes.iter().filter(|byte| **byte != '\r').collect()
    }

    pub fn run(&mut self) -> ! {
        log!("welcome to dnb shell");

        loop {
            let command = self.command();

            log!("command: {:?}", command);

            match command.as_str() {
                "help" => {
                    log!("dnb shell, version 0.1");
                },
                command => {
                },
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn entry() -> ! {
    let mut shell = Shell::new();

    shell.run()
}


