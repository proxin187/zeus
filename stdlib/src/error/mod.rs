

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Error {
    InvalidPath = 1,
    ExpectedFile = 2,
    OutOfBounds = 3,
    OutOfFd = 4,
    NoSuchFd = 5,
    LimitedSpace = 6,
    Barrier = 7,
}

impl Error {
    pub fn from(status: u32) -> Option<Error> {
        match status {
            1 => Some(Error::InvalidPath),
            2 => Some(Error::ExpectedFile),
            3 => Some(Error::OutOfBounds),
            4 => Some(Error::OutOfFd),
            5 => Some(Error::NoSuchFd),
            6 => Some(Error::LimitedSpace),
            7 => Some(Error::Barrier),
            _ => None,
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Error::InvalidPath => f.write_str("invalid path"),
            Error::ExpectedFile => f.write_str("expected file"),
            Error::OutOfBounds => f.write_str("out of bounds"),
            Error::OutOfFd => f.write_str("out of fd"),
            Error::NoSuchFd => f.write_str("no such fd"),
            Error::LimitedSpace => f.write_str("limited space"),
            Error::Barrier => f.write_str("barrier"),
        }
    }
}

impl core::error::Error for Error {}


