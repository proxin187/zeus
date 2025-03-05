use crate::{log, read_csr, write_csr};

pub const MTIME: *mut u64 = 0x200bff8 as *mut u64;
pub const MTIMECMP: *mut u64 = 0x2004000 as *mut u64;


pub fn init() {
    unsafe {
        *MTIMECMP = *MTIME + 0xfff;
    }

    let mut mie = read_csr!("mie", u64);

    mie |= 1 << 7;

    write_csr!("mie", mie);

    log!("schedueler initialized");
}


