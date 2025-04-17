use crate::sys;


pub enum Fork {
    Parent,
    Child,
}

impl From<u32> for Fork {
    fn from(status: u32) -> Fork {
        match status {
            0 => Fork::Parent,
            _ => Fork::Child,
        }
    }
}

#[inline]
pub fn fork() -> Fork {
    let status = unsafe { sys::fork() };

    Fork::from(status)
}


