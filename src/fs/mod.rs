mod vfs;

use crate::drivers::virtio::blk::{VirtioBlk, Mode};
use crate::log;

use alloc::vec::Vec;
use core::cell::OnceCell;

use spin::Mutex;

const MAGIC: [u8; 4] = [0x5a, 0x45, 0x55, 0x53];

static FILE_SYSTEM: Mutex<OnceCell<Fs>> = Mutex::new(OnceCell::new());


#[repr(C)]
pub struct Inode {
    next: Option<usize>,
    len: usize,
}

#[repr(C)]
pub struct Directory {
    entry: usize,
}

#[repr(C)]
pub enum EntryKind {
    File(Inode),
    Directory(Directory),
}

#[repr(C)]
pub struct DirEntry {
    name: [u8; 60],

    // addr points to EntryKind, either a file or another directory
    addr: usize,

    next: Option<usize>,
}

pub struct Fs {
    block: VirtioBlk,
}

impl Fs {
    pub fn new(block: VirtioBlk) -> Fs {
        let mut fs = Fs {
            block,
        };

        fs.init();

        fs
    }

    #[inline]
    fn read_blk(&mut self, sector: u64) -> [u8; 512] {
        let mut buf = [0; 512];

        let status = unsafe { self.block.blk_op(Mode::Read, &mut buf as *mut [u8; 512], sector) };

        match status {
            Ok(()) => buf,
            Err(err) => {
                panic!("failed to read disk: {:?}", err);
            },
        }
    }

    #[inline]
    fn write_blk(&mut self, sector: u64, mut buf: [u8; 512]) {
        let data = unsafe { self.block.blk_op(Mode::Write, &mut buf as *mut [u8; 512], sector) };

        match data {
            Ok(()) => {},
            Err(err) => {
                panic!("failed to read disk: {:?}", err);
            },
        }
    }

    // the first 1024 sectors of the disk represent the zones that say what
    // sectors are free and what sectors are taken, in the default disk there are 65536 sectors in
    // total
    fn init_zones(&mut self) {
        let mut block = [0; 512];

        // this flags the first 1024 + 1 sectors as used
        block[0..130].copy_from_slice(&[0xff; 130]);

        self.write_blk(1, block);

        for sector in 2..1025 {
            let block = [0; 512];

            self.write_blk(0, block);
        }
    }

    fn init(&mut self) {
        if !self.read_blk(0).starts_with(&MAGIC) {
            log!("init fs at '/'");

            let mut superblock = [0; 512];

            superblock[0..4].copy_from_slice(&MAGIC);

            self.write_blk(0, superblock);

            self.init_zones();
        }

        log!("file system setup");
    }

    pub fn touch(&mut self, path: &str) {
    }

    pub fn mkdir(&mut self, path: &str) {
    }

    pub fn list(&mut self, path: &str) {
    }
}

pub fn init(block: VirtioBlk) {
    FILE_SYSTEM.lock().get_or_init(|| Fs::new(block));
}


