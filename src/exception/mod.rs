use crate::{log, write_csr, read_csr};

use core::arch::{naked_asm, asm};

const TIME_OFFSET: u64 = 10000000;

extern "C" {
    static mut __trap_frame: u8;
}

macro_rules! reset_timer {
    ($time:expr) => {
        unsafe {
            asm!(
                "rdtime t0",
                "li t1, {time}",
                "add a0, t0, t1",
                "li a7, 0x54494D45",
                "li a6, 0x00",
                "ecall",
                time = const $time,
            );
        }
    };
}

pub enum Interrupt {
    MachineSoftware,
    SupervisorTimer,
    MachineTimer,
    MachineExternal,
    Unknown,
}

impl From<u64> for Interrupt {
    fn from(mcause: u64) -> Interrupt {
        match mcause & 0xfff {
            3 => Interrupt::MachineSoftware,
            5 => Interrupt::SupervisorTimer,
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
    fn from(mcause: u64) -> Exception {
        match mcause & 0xfff {
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
    fn from(mcause: u64) -> TrapKind {
        match (mcause >> 63) & 1 {
            1 => TrapKind::Interrupt(Interrupt::from(mcause)),
            _ => TrapKind::Exception(Exception::from(mcause)),
        }
    }
}

#[repr(packed, C)]
#[derive(Debug, Clone, Copy)]
pub struct TrapFrame {
    regs: [u64; 32],
}

impl TrapFrame {
    pub const fn new() -> TrapFrame {
        TrapFrame {
            regs: [0; 32],
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn s_handle_trap() {
    let scause = read_csr!("scause", u64);
    let stval = read_csr!("stval", u64);
    let sepc = read_csr!("sepc", u64);

    match TrapKind::from(scause) {
        TrapKind::Interrupt(interrupt) => match interrupt {
            Interrupt::MachineSoftware => {
                log!("machine software interrupt");
            },
            Interrupt::SupervisorTimer => {
                reset_timer!(TIME_OFFSET);

                asm!(
                    "li t0, 32",
                    "csrc sip, t0",
                );

                log!("supervisor timer interrupt");
            },
            Interrupt::MachineTimer => {
                log!("machine timer interrupt");
            },
            Interrupt::MachineExternal => {
                log!("machine external interrupt");
            },
            Interrupt::Unknown => {
                log!("unknown interrupt: cause={}, tval={}, epc={}", scause, stval, sepc);
            },
        },
        TrapKind::Exception(exception) => {
            panic!("exception: {:?}, cause={}, tval={}, epc={}", exception, scause, stval, sepc);

            // the amount of bytes we need here depends on the instruction size, we could maybe
            // only do this for syscalls
            asm!(
                "csrr a0, mepc",
                "addi a0, a0, 0x4",
                "csrw mepc, a0",
            );
        },
    }
}

#[naked]
pub unsafe extern "C" fn trap_entry() {
    naked_asm!(
        "csrw sscratch, a0",

        // TRAPFRAME needs to be the address that we load into
        "la a0, __trap_frame",

        "sd ra, 40(a0)",
        "sd sp, 48(a0)",
        "sd gp, 56(a0)",
        "sd tp, 64(a0)",
        "sd t0, 72(a0)",
        "sd t1, 80(a0)",
        "sd t2, 88(a0)",
        "sd s0, 96(a0)",
        "sd s1, 104(a0)",
        "sd a1, 120(a0)",
        "sd a2, 128(a0)",
        "sd a3, 136(a0)",
        "sd a4, 144(a0)",
        "sd a5, 152(a0)",
        "sd a6, 160(a0)",
        "sd a7, 168(a0)",
        "sd s2, 176(a0)",
        "sd s3, 184(a0)",
        "sd s4, 192(a0)",
        "sd s5, 200(a0)",
        "sd s6, 208(a0)",
        "sd s7, 216(a0)",
        "sd s8, 224(a0)",
        "sd s9, 232(a0)",
        "sd s10, 240(a0)",
        "sd s11, 248(a0)",
        "sd t3, 256(a0)",
        "sd t4, 264(a0)",
        "sd t5, 272(a0)",
        "sd t6, 280(a0)",

        // save the a0 register
        "csrr t0, sscratch",
        "sd t0, 112(a0)",

        // this is the old code, it saves on the stack which is bad
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

        "call s_handle_trap",

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

    unsafe {
        asm!("csrsi sstatus, 2");
    }

    reset_timer!(TIME_OFFSET);

    unsafe {
        asm!(
            "li t1, 32",
            "csrs sie, t1",
        );
    }

    log!("exception handler set and supervisor interrupt enabled");
}


