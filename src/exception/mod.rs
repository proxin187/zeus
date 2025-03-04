use core::arch::{naked_asm, asm};

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
pub unsafe extern "C" fn m_handle_trap() {
    let scause = read_csr!("scause", u64);
    let stval = read_csr!("stval", u64);
    let pc = read_csr!("sepc", u64);

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
            // TODO: we might have to consider dropping opensbi as we really dont need it because
            // it drops us into supervisor mode when it would be easier if we were just in machine
            // mode

            log!("exception: {:?}, scause={}, stval={}, pc={}", exception, scause, stval, pc);

            asm!(
                "csrr a0, sepc",
                "addi a0, a0, 0x4",
                "csrw sepc, a0",
            );
        },
    }
}

#[naked]
pub unsafe extern "C" fn trap_entry() {
    naked_asm!(
        "addi sp, sp, -256",

        "sd ra, 0(sp)",
        "sd sp, 8(sp)",
        "sd gp, 16(sp)",
        "sd tp, 24(sp)",
        "sd t0, 32(sp)",
        "sd t1, 40(sp)",
        "sd t2, 48(sp)",
        "sd s0, 56(sp)",
        "sd s1, 64(sp)",
        "sd a0, 72(sp)",
        "sd a1, 80(sp)",
        "sd a2, 88(sp)",
        "sd a3, 96(sp)",
        "sd a4, 104(sp)",
        "sd a5, 112(sp)",
        "sd a6, 120(sp)",
        "sd a7, 128(sp)",
        "sd s2, 136(sp)",
        "sd s3, 144(sp)",
        "sd s4, 152(sp)",
        "sd s5, 160(sp)",
        "sd s6, 168(sp)",
        "sd s7, 176(sp)",
        "sd s8, 184(sp)",
        "sd s9, 192(sp)",
        "sd s10, 200(sp)",
        "sd s11, 208(sp)",
        "sd t3, 216(sp)",
        "sd t4, 224(sp)",
        "sd t5, 232(sp)",
        "sd t6, 240(sp)",

        "call m_handle_trap",

        "ld ra, 0(sp)",
        "ld sp, 8(sp)",
        "ld gp, 16(sp)",
        "ld t0, 32(sp)",
        "ld t1, 40(sp)",
        "ld t2, 48(sp)",
        "ld s0, 56(sp)",
        "ld s1, 64(sp)",
        "ld a0, 72(sp)",
        "ld a1, 80(sp)",
        "ld a2, 88(sp)",
        "ld a3, 96(sp)",
        "ld a4, 104(sp)",
        "ld a5, 112(sp)",
        "ld a6, 120(sp)",
        "ld a7, 128(sp)",
        "ld s2, 136(sp)",
        "ld s3, 144(sp)",
        "ld s4, 152(sp)",
        "ld s5, 160(sp)",
        "ld s6, 168(sp)",
        "ld s7, 176(sp)",
        "ld s8, 184(sp)",
        "ld s9, 192(sp)",
        "ld s10, 200(sp)",
        "ld s11, 208(sp)",
        "ld t3, 216(sp)",
        "ld t4, 224(sp)",
        "ld t5, 232(sp)",
        "ld t6, 240(sp)",

        "addi sp, sp, 256",

        "sret",
    );
}

pub fn init() {
    write_csr!("stvec", trap_entry);

    log!("exception handler set");
}


