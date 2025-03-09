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
pub unsafe extern "C" fn s_handle_trap(trapframe: &TrapFrame) {
    let test = trapframe;

    let scause = read_csr!("scause", u64);
    let stval = read_csr!("stval", u64);
    let sepc = read_csr!("sepc", u64);

    log!("test: {:?}", test);

    match TrapKind::from(scause) {
        TrapKind::Interrupt(interrupt) => match interrupt {
            Interrupt::MachineSoftware => {
                log!("machine software interrupt");
            },
            Interrupt::SupervisorTimer => {
                // here we will have to do a context switch

                // it is important that we dont reset the timer before we are done with the context
                // switch in order to not get an timer interrupt while inside the kernel
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
        // save a0 into scratch register
        "csrw sscratch, a0",

        // load address of our trapframe
        "la a0, __trap_frame",

        // save registers into trapframe
        "sd ra, 0(a0)",
        "sd sp, 8(a0)",
        "sd gp, 16(a0)",
        "sd tp, 24(a0)",
        "sd t0, 32(a0)",
        "sd t1, 40(a0)",
        "sd t2, 48(a0)",
        "sd s0, 56(a0)",
        "sd s1, 64(a0)",

        // we skip one here because this is where a0 is
        "sd a1, 80(a0)",
        "sd a2, 88(a0)",
        "sd a3, 96(a0)",
        "sd a4, 104(a0)",
        "sd a5, 112(a0)",
        "sd a6, 120(a0)",
        "sd a7, 128(a0)",
        "sd s2, 136(a0)",
        "sd s3, 144(a0)",
        "sd s4, 152(a0)",
        "sd s5, 160(a0)",
        "sd s6, 168(a0)",
        "sd s7, 176(a0)",
        "sd s8, 184(a0)",
        "sd s9, 192(a0)",
        "sd s10, 200(a0)",
        "sd s11, 208(a0)",
        "sd t3, 216(a0)",
        "sd t4, 224(a0)",
        "sd t5, 232(a0)",
        "sd t6, 240(a0)",

        // save the a0 register
        "csrr t0, sscratch",
        "sd t0, 72(a0)",

        // load the kernel trap handling stack
        "la sp, __kstack",

        // the trapframe address is already in a0 before calling
        "call s_handle_trap",

        // load address of our trapframe
        "la a0, __trap_frame",

        // save registers into trapframe
        "ld ra, 0(a0)",
        "ld sp, 8(a0)",
        "ld gp, 16(a0)",
        "ld tp, 24(a0)",
        "ld t0, 32(a0)",
        "ld t1, 40(a0)",
        "ld t2, 48(a0)",
        "ld s0, 56(a0)",
        "ld s1, 64(a0)",

        // we skip one here because this is where a0 is
        "ld a1, 80(a0)",
        "ld a2, 88(a0)",
        "ld a3, 96(a0)",
        "ld a4, 104(a0)",
        "ld a5, 112(a0)",
        "ld a6, 120(a0)",
        "ld a7, 128(a0)",
        "ld s2, 136(a0)",
        "ld s3, 144(a0)",
        "ld s4, 152(a0)",
        "ld s5, 160(a0)",
        "ld s6, 168(a0)",
        "ld s7, 176(a0)",
        "ld s8, 184(a0)",
        "ld s9, 192(a0)",
        "ld s10, 200(a0)",
        "ld s11, 208(a0)",
        "ld t3, 216(a0)",
        "ld t4, 224(a0)",
        "ld t5, 232(a0)",
        "ld t6, 240(a0)",

        "ld a0, 72(a0)",

        // sepc will be updated inside the traphandler and not here

        // return to the program
        "sret",
    );
}

pub fn init() {
    write_csr!("stvec", trap_entry);

    log!("exception handler initialized");
}

pub fn init_timer() {
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

    log!("timer initialized");
}


