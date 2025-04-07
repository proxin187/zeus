use crate::exception::{__trap_frame, TrapFrame};
use crate::{write_csr, log, process};
use crate::fs::vfs;

use core::slice;


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
            let status = vfs::lock(|vfs| {
                let bytes = unsafe { slice::from_raw_parts(trapframe.regs[10] as *const u8, trapframe.regs[11] as usize) };

                vfs.write(trapframe.regs[12] as u32, bytes)
            });

            // TODO: return the status to the caller
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


