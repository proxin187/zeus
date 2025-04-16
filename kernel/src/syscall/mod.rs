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
    // a7: syscall number
    let syscall = Syscall::from(trapframe.regs[16]);

    // log!("syscall: {:?}", syscall);

    match syscall {
        Syscall::Write => {
            // a6: fd, a5: len, a4: addr -> a7: status

            let status = vfs::lock(|vfs| {
                let bytes = unsafe { slice::from_raw_parts(trapframe.regs[13] as *const u8, trapframe.regs[14] as usize) };

                vfs.write(trapframe.regs[15] as u32, bytes)
            });

            unsafe {
                __trap_frame.regs[16] = status.map_err(|err| err as u64).err().unwrap_or(0);
            }
        },
        Syscall::Read => {
            // a6: fd, a5: len -> a7: status, a6: addr

            match vfs::lock(|vfs| vfs.read(trapframe.regs[15] as u32, trapframe.regs[14] as u32)) {
                Ok(bytes) => {
                    unsafe {
                        __trap_frame.regs[16] = 0;

                        __trap_frame.regs[15] = bytes.leak() as *mut [u8] as *const u8 as u64;
                    }
                },
                Err(err) => {
                    unsafe {
                        __trap_frame.regs[16] = err as u64;
                    }
                },
            }
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


