mod vfs;

use crate::drivers::virtio::blk::{VirtioBlk, Mode};
use crate::log;

use alloc::collections::BTreeMap;

use core::cell::OnceCell;
use core::mem;
use core::ptr;

use spin::Mutex;

static FILE_SYSTEM: Mutex<OnceCell<Fs>> = Mutex::new(OnceCell::new());


#[repr(C)]
pub struct Header {
    magic: u32,
    entries: u32,
}

impl Header {
    pub fn new(block: [u8; 512]) -> Header {
        unsafe {
            ptr::read(block.as_ptr() as *const Header)
        }
    }
}

#[repr(C)]
pub struct Cluster {
    next: Option<u32>,
    len: u32,
    data: [u8; 492],
}

#[repr(C)]
pub struct DirEntry {
    name: [u8; 56],
    addr: Option<u32>,
}

pub struct Blocks {
    driver: VirtioBlk,
}

impl Blocks {
    fn read(&mut self, sector: u64) -> [u8; 512] {
        let mut buf = [0; 512];

        let status = unsafe { self.driver.blk_op(Mode::Read, &mut buf as *mut [u8; 512], sector) };

        match status {
            Ok(()) => buf,
            Err(err) => {
                panic!("failed to read disk: {:?}", err);
            },
        }
    }

    fn write(&mut self, sector: u64, mut buf: [u8; 512]) {
        let data = unsafe { self.driver.blk_op(Mode::Write, &mut buf as *mut [u8; 512], sector) };

        match data {
            Ok(()) => {},
            Err(err) => {
                panic!("failed to read disk: {:?}", err);
            },
        }
    }
}

pub struct Fs {
    blocks: Blocks,
    cache: BTreeMap<[u8; 60], Option<usize>>,
}

impl Fs {
    pub fn new(driver: VirtioBlk) -> Fs {
        let mut fs = Fs {
            blocks: Blocks {
                driver,
            },
            cache: BTreeMap::new(),
        };

        fs.init();

        fs
    }

    fn load(&mut self, header: Header) {
        log!("loading entries={}, sectors={}", header.entries, header.entries * mem::size_of::<DirEntry>() as u32 / 512);

        for sector in 0..header.entries * mem::size_of::<DirEntry>() as u32 / 512 {
            let block = self.blocks.read(sector as u64);
        }
    }

    fn setup(&mut self) {
        log!("setup started");

        let mut superblock = [0; 512];

        unsafe {
            (superblock.as_mut_ptr() as *mut Header).write(Header { magic: 0x5a455553, entries: 0 });
        }

        self.blocks.write(0, superblock);
    }

    fn init(&mut self) {
        let block = self.blocks.read(0);
        let header = Header::new(block);

        match header.magic {
            0x5a455553 => self.load(header),
            _ => self.setup(),
        }
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


