use crate::{log, process};


pub enum Syscall {
    Write,
    Read,
    Exit,
}

impl From<u64> for Syscall {
    fn from(value: u64) -> Syscall {
        match value {
            0 => Syscall::Write,
            1 => Syscall::Read,
            2 => Syscall::Exit,
            _ => panic!("unknown syscall: {}", value),
        }
    }
}

#[no_mangle]
#[link_section = ".text.syscall"]
pub unsafe extern "C" fn syscall(a0: u64) {
    match Syscall::from(a0) {
        Syscall::Write => {
        },
        Syscall::Read => {
        },
        Syscall::Exit => {
            log!("exiting");

            process::exit();
        },
    }
}


