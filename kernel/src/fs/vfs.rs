use crate::drivers::virtio::blk::VirtioBlk;

use super::{Fs, Error};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;


pub struct Descriptor {
    name: [u8; 56],
    offset: u32,
}

impl Descriptor {
    pub fn new(name: [u8; 56], offset: u32) -> Descriptor {
        Descriptor {
            name,
            offset,
        }
    }
}

pub struct Vfs {
    fs: Fs,
    descriptors: BTreeMap<u32, Descriptor>,
}

impl Vfs {
    pub fn new(driver: VirtioBlk) -> Vfs {
        let mut descriptors = BTreeMap::new();

        descriptors.insert(u32::MIN, Descriptor::new([0; 56], 0));

        descriptors.insert(u32::MAX, Descriptor::new([0; 56], 0));

        Vfs {
            fs: Fs::new(driver),
            descriptors,
        }
    }

    pub fn open(&mut self, name: [u8; 56]) -> Result<u32, Error> {
        let fd = self.descriptors.iter()
            .map_windows(|[(a, _), (b, _)]| (**a != *b - 1).then(|| *b - 1))
            .flatten()
            .next()
            .ok_or(Error::OutOfFd)?;

        self.descriptors.insert(fd, Descriptor::new(name, 0));

        Ok(fd)
    }

    pub fn seek(&mut self, fd: u32, offset: u32) -> Result<(), Error> {
        match self.descriptors.get_mut(&fd) {
            Some(descriptor) => {
                descriptor.offset = offset;

                Ok(())
            },
            None => Err(Error::NoSuchFd),
        }
    }

    pub fn read(&mut self, fd: u32, bytes: u32) -> Result<Vec<u8>, Error> {
        match self.descriptors.get_mut(&fd) {
            Some(descriptor) => {
                descriptor.offset += bytes;

                self.fs.read(descriptor.name, descriptor.offset - bytes..descriptor.offset)
            },
            None => Err(Error::NoSuchFd),
        }
    }
}


