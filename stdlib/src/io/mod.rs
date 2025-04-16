use crate::sys;


pub static mut STDIO: Stdio = Stdio::new();

pub struct Stdio;

impl Stdio {
    const fn new() -> Stdio { Stdio }
}

impl core::fmt::Write for Stdio {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        unsafe {
            if let Err(err) = sys::write(sys::STDIO, string.len() as u32, string.as_ptr()) {
                panic!("stdio failed to write: {:?}", err);
            }
        }

        Ok(())
    }
}

#[allow(static_mut_refs)]

pub fn _print(args: core::fmt::Arguments) -> core::fmt::Result {
    unsafe {
        core::fmt::write(&mut STDIO, args)
    }
}


