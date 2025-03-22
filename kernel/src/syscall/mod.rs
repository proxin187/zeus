use crate::exception::{__trap_frame, TrapFrame};
use crate::{write_csr, log, process};


#[derive(Debug)]
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
            93 => Syscall::Exit,
            _ => panic!("unknown syscall: {}", value),
        }
    }
}

pub fn syscall(trapframe: &TrapFrame) {
    let syscall = Syscall::from(trapframe.regs[16]);

    log!("syscall: {:?}", syscall);

    match syscall {
        Syscall::Write => {
        },
        Syscall::Read => {
        },
        Syscall::Exit => {
            let context = process::exit();

            unsafe {
                __trap_frame = context.frame;
            }

            write_csr!("sepc", context.epc - 4);
        },
    }
}


