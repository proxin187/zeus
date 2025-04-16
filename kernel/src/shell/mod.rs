use alloc::string::String;
use alloc::vec::Vec;

use stdlib::{sys, print, println};


pub struct Shell {
    cwd: [u8; 56],
}

impl Shell {
    pub fn new() -> Shell {
        Shell {
            cwd: [0; 56],
        }
    }

    fn readline(&self) -> String {
        let mut bytes: Vec<char> = Vec::new();

        print!("[proxin@proxin home]$ ");

        while !bytes.ends_with(&['\n']) && !bytes.ends_with(&['\r']) {
            if let Ok(character) = unsafe { sys::read(sys::STDIO, 1).map(|ptr| *ptr as char) } {
                print!("{}", character);

                bytes.push(character);
            }
        }

        print!("\n");

        bytes.iter().filter(|byte| **byte != '\r').collect()
    }

    pub fn run(&mut self) -> ! {
        println!("[dnb] logged in with usermode");

        loop {
            let command = self.readline();

            match command.as_str() {
                "help" => {
                    println!("[dnb] shell, version 0.1");
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


