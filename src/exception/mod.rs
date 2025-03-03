use core::arch::asm;

use crate::{log, write_csr, read_csr};


pub enum Interrupt {
    MachineSoftware,
    MachineTimer,
    MachineExternal,
    Unknown,
}

impl From<u64> for Interrupt {
    fn from(scause: u64) -> Interrupt {
        match scause & 0xfff {
            3 => Interrupt::MachineSoftware,
            7 => Interrupt::MachineTimer,
            11 => Interrupt::MachineExternal,
            _ => Interrupt::Unknown,
        }
    }
}

#[derive(Debug)]
pub enum Exception {
    IllegalInstruction,
    SyscallUser,
    SyscallSupervisor,
    SyscallMachine,
    Unknown,
}

impl From<u64> for Exception {
    fn from(scause: u64) -> Exception {
        match scause & 0xfff {
            2 => Exception::IllegalInstruction,
            8 => Exception::SyscallUser,
            9 => Exception::SyscallSupervisor,
            11 => Exception::SyscallMachine,
            _ => Exception::Unknown,
        }
    }
}

pub enum TrapKind {
    Interrupt(Interrupt),
    Exception(Exception),
}

impl From<u64> for TrapKind {
    fn from(scause: u64) -> TrapKind {
        match (scause >> 63) & 1 {
            1 => TrapKind::Interrupt(Interrupt::from(scause)),
            _ => TrapKind::Exception(Exception::from(scause)),
        }
    }
}

#[repr(packed, C)]
#[derive(Debug)]
pub struct TrapFrame {
    registers: [u64; 31],
}

#[no_mangle]
pub unsafe extern "C" fn handle_trap(trap: &TrapFrame) {
    let scause = read_csr!("scause", u64);
    let stval = read_csr!("stval", u64);
    let pc = read_csr!("sepc", u64);

    log!("exception: {:?}, scause={}, stval={}, pc={}", trap, scause, stval, pc);

    return;
    match TrapKind::from(scause) {
        TrapKind::Interrupt(interrupt) => match interrupt {
            Interrupt::MachineSoftware => {
                log!("machine software interrupt");
            },
            Interrupt::MachineTimer => {
                log!("timer interrupt");
            },
            Interrupt::MachineExternal => {
                log!("machine external interrupt");
            },
            Interrupt::Unknown => {
                log!("unknown interrupt: scause={}, stval={}, pc={}", scause, stval, pc);
            },
        },
        TrapKind::Exception(exception) => {
            panic!("exception: {:?}, scause={}, stval={}, pc={}", exception, scause, stval, pc);
        },
    }
}

#[naked]
pub unsafe extern "C" fn trap_entry() {
    // TODO: this fails to return most likely because we are in 64bit but this is for 32bit
    asm!(
        "csrw sscratch, sp",
        "addi sp, sp, -4 * 31",
        "sw ra,  4 * 0(sp)",
        "sw gp,  4 * 1(sp)",
        "sw tp,  4 * 2(sp)",
        "sw t0,  4 * 3(sp)",
        "sw t1,  4 * 4(sp)",
        "sw t2,  4 * 5(sp)",
        "sw t3,  4 * 6(sp)",
        "sw t4,  4 * 7(sp)",
        "sw t5,  4 * 8(sp)",
        "sw t6,  4 * 9(sp)",
        "sw a0,  4 * 10(sp)",
        "sw a1,  4 * 11(sp)",
        "sw a2,  4 * 12(sp)",
        "sw a3,  4 * 13(sp)",
        "sw a4,  4 * 14(sp)",
        "sw a5,  4 * 15(sp)",
        "sw a6,  4 * 16(sp)",
        "sw a7,  4 * 17(sp)",
        "sw s0,  4 * 18(sp)",
        "sw s1,  4 * 19(sp)",
        "sw s2,  4 * 20(sp)",
        "sw s3,  4 * 21(sp)",
        "sw s4,  4 * 22(sp)",
        "sw s5,  4 * 23(sp)",
        "sw s6,  4 * 24(sp)",
        "sw s7,  4 * 25(sp)",
        "sw s8,  4 * 26(sp)",
        "sw s9,  4 * 27(sp)",
        "sw s10, 4 * 28(sp)",
        "sw s11, 4 * 29(sp)",

        "csrr a0, sscratch",
        "sw a0, 4 * 30(sp)",

        "mv a0, sp",
        "call handle_trap",

        "lw ra,  4 * 0(sp)",
        "lw gp,  4 * 1(sp)",
        "lw tp,  4 * 2(sp)",
        "lw t0,  4 * 3(sp)",
        "lw t1,  4 * 4(sp)",
        "lw t2,  4 * 5(sp)",
        "lw t3,  4 * 6(sp)",
        "lw t4,  4 * 7(sp)",
        "lw t5,  4 * 8(sp)",
        "lw t6,  4 * 9(sp)",
        "lw a0,  4 * 10(sp)",
        "lw a1,  4 * 11(sp)",
        "lw a2,  4 * 12(sp)",
        "lw a3,  4 * 13(sp)",
        "lw a4,  4 * 14(sp)",
        "lw a5,  4 * 15(sp)",
        "lw a6,  4 * 16(sp)",
        "lw a7,  4 * 17(sp)",
        "lw s0,  4 * 18(sp)",
        "lw s1,  4 * 19(sp)",
        "lw s2,  4 * 20(sp)",
        "lw s3,  4 * 21(sp)",
        "lw s4,  4 * 22(sp)",
        "lw s5,  4 * 23(sp)",
        "lw s6,  4 * 24(sp)",
        "lw s7,  4 * 25(sp)",
        "lw s8,  4 * 26(sp)",
        "lw s9,  4 * 27(sp)",
        "lw s10, 4 * 28(sp)",
        "lw s11, 4 * 29(sp)",
        "lw sp,  4 * 30(sp)",
        "sret",
        options(noreturn),
    );
}

pub fn init() {
    write_csr!("stvec", trap_entry);

    log!("exception handler set");
}


