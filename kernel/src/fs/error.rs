

#[repr(u8)]
#[derive(Debug)]
pub enum Error {
    InvalidPath,
    ExpectedFile,
    OutOfBounds,
    LimitedSpace,
}


