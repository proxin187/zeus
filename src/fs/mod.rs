mod vfs;

use crate::drivers::virtio::blk::VirtioBlk;

use alloc::vec::Vec;
use core::cell::OnceCell;

use spin::Mutex;

static FILE_SYSTEM: Mutex<OnceCell<Fs>> = Mutex::new(OnceCell::new());


pub struct File {
}

pub struct Directory {
}

pub struct Header {
}

pub struct Fs {
    block: VirtioBlk,
}

impl Fs {
    pub fn new(block: VirtioBlk) -> Fs {
        Fs {
            block,
        }
    }
}

pub fn init(block: VirtioBlk) {
    FILE_SYSTEM.lock().get_or_init(|| Fs::new(block));
}


