use crate::drivers::virtio::blk::VirtioBlk;
use crate::drivers::uart;

use super::{Fs, FS};

use core::cell::OnceCell;

use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::vec::Vec;

use stdlib::error::Error;
use spin::Mutex;

static VFS: Mutex<OnceCell<Vfs>> = Mutex::new(OnceCell::new());


trait Descriptor {
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error>;
    fn read(&mut self, bytes: u32) -> Result<Vec<u8>, Error>;
    fn seek(&mut self, offset: u32);
}

pub struct Fd {
    name: [u8; 56],
    offset: u32,
}

impl Fd {
    pub fn new(name: [u8; 56], offset: u32) -> Fd {
        Fd {
            name,
            offset,
        }
    }
}

impl Descriptor for Fd {
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.offset += bytes.len() as u32;

        let mut lock = FS.lock();

        lock.get_mut()
            .expect("uninitialized file system")
            .write(self.name, self.offset - bytes.len() as u32, bytes)
    }

    fn read(&mut self, bytes: u32) -> Result<Vec<u8>, Error> {
        self.offset += bytes;

        let mut lock = FS.lock();

        lock.get_mut()
            .expect("uninitialized file system")
            .read(self.name, self.offset - bytes..self.offset)
    }

    fn seek(&mut self, offset: u32) {
        self.offset = offset;
    }
}

pub struct Stdio;

impl Descriptor for Stdio {
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for byte in bytes {
            unsafe {
                uart::UART.write(*byte);
            }
        }

        Ok(())
    }

    fn read(&mut self, _: u32) -> Result<Vec<u8>, Error> {
        todo!("read from stdin");
    }

    fn seek(&mut self, _: u32) {}
}

pub struct Barrier;

impl Descriptor for Barrier {
    fn write(&mut self, bytes: &[u8]) -> Result<(), Error> {
        Err(Error::Barrier)
    }

    fn read(&mut self, _: u32) -> Result<Vec<u8>, Error> {
        Err(Error::Barrier)
    }

    fn seek(&mut self, _: u32) {}
}

pub struct Vfs {
    descriptors: BTreeMap<u32, Box<dyn Descriptor + Send + Sync>>,
}

impl Vfs {
    pub fn new() -> Vfs {
        let mut descriptors: BTreeMap<u32, Box<dyn Descriptor + Send + Sync>> = BTreeMap::new();

        descriptors.insert(0, Box::new(Stdio));

        descriptors.insert(u32::MAX, Box::new(Barrier));

        Vfs {
            descriptors,
        }
    }

    pub fn open(&mut self, name: [u8; 56]) -> Result<u32, Error> {
        let fd = self.descriptors.iter()
            .map_windows(|[(a, _), (b, _)]| (**a != *b - 1).then(|| *b - 1))
            .flatten()
            .next()
            .ok_or(Error::OutOfFd)?;

        self.descriptors.insert(fd, Box::new(Fd::new(name, 0)));

        Ok(fd)
    }

    pub fn seek(&mut self, fd: u32, offset: u32) -> Result<(), Error> {
        match self.descriptors.get_mut(&fd) {
            Some(descriptor) => {
                descriptor.seek(offset);

                Ok(())
            },
            None => Err(Error::NoSuchFd),
        }
    }

    pub fn read(&mut self, fd: u32, bytes: u32) -> Result<Vec<u8>, Error> {
        match self.descriptors.get_mut(&fd) {
            Some(descriptor) => descriptor.read(bytes),
            None => Err(Error::NoSuchFd),
        }
    }

    pub fn write(&mut self, fd: u32, bytes: &[u8]) -> Result<(), Error> {
        match self.descriptors.get_mut(&fd) {
            Some(descriptor) => descriptor.write(bytes),
            None => Err(Error::NoSuchFd),
        }
    }
}

pub fn init(driver: VirtioBlk) {
    FS.lock().get_or_init(|| Fs::new(driver));

    VFS.lock().get_or_init(|| Vfs::new());
}

pub fn lock<T, F: Fn(&mut Vfs) -> Result<T, Error>>(f: F) -> Result<T, Error> {
    match VFS.lock().get_mut() {
        Some(vfs) => f(vfs),
        None => panic!("virtual file system is uninitialized"),
    }
}


