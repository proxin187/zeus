use core::arch::asm;


// TODO: implement the syscalls into the standard library

#[inline]
pub unsafe fn write(addr: *const u8, len: u32, fd: u32) {
    unsafe {
        asm!(
            "li a0, 1",
            "li a7, 93",
            "ecall",
        );
    }
}

#[inline]
pub unsafe fn read(fd: u32, len: u32) {
}

#[inline]
pub unsafe fn exit() {
    unsafe {
        asm!(
            "li a7, 93",
            "ecall",
        );
    }
}


