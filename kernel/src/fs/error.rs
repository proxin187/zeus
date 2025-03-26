

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Error {
    InvalidPath,
    ExpectedFile,
    OutOfBounds,
    LimitedSpace,
}


