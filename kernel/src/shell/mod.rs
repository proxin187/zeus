use alloc::string::String;
use alloc::vec::Vec;

use stdlib::{sys, print, println};
use stdlib::process::{self, Fork};


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
                match character {
                    '\x7f' => {
                        if let Some(_) = bytes.pop() {
                            print!("\x1b[1D\x1b[J");
                        }
                    },
                    _ => {
                        print!("{}", character);

                        bytes.push(character);
                    },
                }
            }
        }

        print!("\n");

        bytes.iter().filter(|byte| **byte != '\r').collect()
    }

    fn spawn(&mut self, command: String) {
        match process::fork() {
            Fork::Parent => {
                // TODO: we get here but it panics when trying to run the child
                //
                // it blames an illegal instruction which most likely means that we jump to an
                // illegal instruction

                println!("hello from parent");

                loop {}
            },
            Fork::Child => {
                // TODO: it panics before this

                println!("hello from child");

                loop {}
            },
        }
    }

    pub fn run(&mut self) -> ! {
        println!("[dnb] FluxOS (0.1-flux-riscv64 tty)");

        loop {
            let command = self.readline();

            match command.as_str() {
                "help" => {
                    println!("[dnb] shell, version 0.1");
                },
                _ => {
                    self.spawn(command);
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


