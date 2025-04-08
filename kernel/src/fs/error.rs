

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Error {
    InvalidPath = 1,
    ExpectedFile = 2,
    OutOfBounds = 3,
    OutOfFd = 4,
    NoSuchFd = 5,
    LimitedSpace = 6,
}


