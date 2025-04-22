use crate::error::Error;

use core::arch::asm;

pub const STDIO: u32 = 0;


#[inline]
pub unsafe fn write(fd: u32, len: u32, addr: *const u8) -> Result<(), Error> {
    unsafe {
        let status: u32;

        asm!(
            "ecall",
            in("a7") 0,
            in("a6") fd,
            in("a5") len,
            in("a4") addr,
            lateout("a7") status,
        );

        Error::from(status).map_or(Ok(()), |err| Err(err))
    }
}

#[inline]
pub unsafe fn read(fd: u32, len: u32) -> Result<*const u8, Error> {
    unsafe {
        let status: u32;
        let addr: *const u8;

        asm!(
            "ecall",
            in("a7") 1,
            in("a6") fd,
            in("a5") len,
            lateout("a7") status,
            lateout("a6") addr,
        );

        Error::from(status).map_or(Ok(addr), |err| Err(err))
    }
}

#[inline]
pub unsafe fn fork() -> u32 {
    unsafe {
        let status: u32;

        asm!(
            "ecall",
            in("a7") 57,
            lateout("a7") status,
        );

        status
    }
}

#[inline]
pub unsafe fn alloc(size: usize, align: usize) -> *mut u8 {
    unsafe {
        let addr: *mut u8;

        asm!(
            "ecall",
            in("a7") 61,
            in("a6") size,
            in("a5") align,
            lateout("a7") addr,
        );

        addr
    }
}

#[inline]
pub unsafe fn dealloc(size: usize, align: usize, addr: *mut u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") 62,
            in("a6") size,
            in("a5") align,
            in("a4") addr,
        );
    }
}

#[inline]
pub unsafe fn exit() {
    unsafe {
        asm!(
            "ecall",
            in("a7") 93,
        );
    }
}


