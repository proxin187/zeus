use crate::exception::{__trap_frame, TrapFrame};
use crate::{write_csr, process};
use crate::fs::vfs;
use crate::memory;

use core::alloc::{GlobalAlloc, Layout};
use core::slice;


#[derive(Debug, PartialEq)]
pub enum Syscall {
    Write,
    Read,
    Spawn,
    Alloc,
    Dealloc,
    Exit,
}

impl From<u64> for Syscall {
    fn from(value: u64) -> Syscall {
        match value {
            0 => Syscall::Write,
            1 => Syscall::Read,
            60 => Syscall::Spawn,
            61 => Syscall::Alloc,
            62 => Syscall::Dealloc,
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
        Syscall::Spawn => {
        },
        Syscall::Alloc | Syscall::Dealloc => {
            match Layout::from_size_align(trapframe.regs[15] as usize, trapframe.regs[14] as usize) {
                Ok(layout) => {
                    unsafe {
                        if syscall == Syscall::Alloc {
                            // a6: size, a5: align -> a7: addr

                            __trap_frame.regs[16] = memory::ALLOC.alloc(layout) as u64;
                        } else {
                            // a6: size, a5: align, a4: addr -> none

                            memory::ALLOC.dealloc(trapframe.regs[13] as *mut u8, layout);
                        }
                    }
                },
                Err(_) => {
                    unsafe {
                        __trap_frame.regs[16] = 0;
                    }
                },
            }
        },
        Syscall::Exit => {
            // none -> none

            let context = process::lock(|mut processes| processes.exit());

            unsafe {
                __trap_frame = context.frame;
            }

            write_csr!("sepc", context.epc - 4);
        },
    }
}


