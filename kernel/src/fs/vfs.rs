use crate::drivers::virtio::blk::VirtioBlk;

use super::{Fs, Error};

use alloc::collections::BTreeMap;


pub struct Descriptor {
    name: [u8; 56],
    offset: usize,
}

impl Descriptor {
    pub fn new(name: [u8; 56]) -> Descriptor {
        Descriptor {
            name,
            offset: 0,
        }
    }
}

pub struct Vfs {
    fs: Fs,
    descriptors: BTreeMap<u32, Descriptor>,
}

impl Vfs {
    pub fn new(driver: VirtioBlk) -> Vfs {
        Vfs {
            fs: Fs::new(driver),
            descriptors: BTreeMap::new(),
        }
    }

    pub fn open(&mut self, name: [u8; 56]) -> Result<u32, Error> {
        let fd = self.descriptors.iter()
            .map_windows(|[(a, _), (b, _)]| (**a != *b - 1).then(|| *b - 1))
            .flatten()
            .next()
            .ok_or(Error::OutOfFd)?;

        self.descriptors.insert(fd, Descriptor::new());

        Ok(fd)
    }

    pub fn read(&mut self, fd: u32) -> Result<(), Error> {
        match self.descriptors.get(&fd) {
            Some(descriptor) => {

                Ok(())
            },
            None => Err(Error::NoSuchFd),
        }
    }
}


