mod error;
mod vfs;

use error::Error;

use crate::drivers::virtio::blk::{VirtioBlk, Mode};
use crate::log;

use alloc::collections::BTreeMap;

use core::cell::OnceCell;
use core::ops::Range;
use core::mem;
use core::ptr;

use spin::Mutex;

static FILE_SYSTEM: Mutex<OnceCell<Fs>> = Mutex::new(OnceCell::new());

macro_rules! decode {
    ($value:expr) => {
        unsafe {
            ptr::read($value as *const [u8] as *const _)
        }
    };
}

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
#[derive(Debug)]
pub struct Cluster {
    next: Option<u32>,
    len: u32,
    data: [u8; 492],
}

#[repr(C)]
#[derive(Debug)]
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
    cache: BTreeMap<[u8; 56], Option<u32>>,
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

    fn cache(&mut self, block: &[u8]) {
        for chunk in block.chunks(mem::size_of::<DirEntry>()) {
            if chunk.iter().all(|byte| *byte == 0) {
                break;
            } else {
                let entry: DirEntry = decode!(chunk);

                log!("entry: {:?}", entry);

                self.cache.insert(entry.name, entry.addr);
            }
        }
    }

    fn load(&mut self, header: Header) {
        log!("loading entries={}, sectors={}", header.entries, 1 + header.entries * mem::size_of::<DirEntry>() as u32 / 512);

        for sector in 0..1 + header.entries * mem::size_of::<DirEntry>() as u32 / 512 {
            let block = self.blocks.read(1 + sector as u64);

            self.cache(&block);
        }
    }

    fn init(&mut self) {
        let block = self.blocks.read(0);
        let header = Header::new(block);

        match header.magic {
            0x5a455553 => self.load(header),
            _ => {
                panic!("invalid tndfs partition");
            },
        }
    }

    // TODO: finish this
    fn read_cluster(&mut self, mut cluster: Cluster, range: Range<u32>, buf: &mut [u8]) -> Result<(), Error> {
        let mut count: u32 = 0;

        while let Some(next) = cluster.next {
            log!("cluster: {:?}", cluster);

            // this checks if the start is inside the current cluster, eg. bigger than the start
            // offset and smaller than the end offset
            if range.start >= count && range.start < count + cluster.len {
                // TODO: finish this
                buf[count as usize..count as usize + cluster.len as usize].copy_from_slice(&cluster.data[range.start as usize - count as usize..cluster.len as usize]);
                // we are in the first cluster
            } else if range.end >= count && range.end < count + cluster.len {
                buf[count as usize..count as usize + cluster.len as usize].copy_from_slice(&cluster.data[..cluster.len as usize]);
                // we are in the last cluster
            } else if range.start < count && range.end >= count {
                // we are in a middle cluster
            } else {
                return Ok(());
            }

            count += cluster.len;

            cluster = decode!(&self.blocks.read(next as u64));
        }

        Err(Error::OutOfBounds)
    }

    // the simplest way to do this would be to simply iterate over the file and keep a count of the
    // index and iterate until we hit the start
    //
    // another way to do this would be to iterate over just the clusters and check if the count is
    // inside else advance onto the next cluster and so on.
    pub fn read(&mut self, range: Range<u32>, path: [u8; 56], buf: &mut [u8]) -> Result<(), Error> {
        match self.cache.get(&path) {
            Some(addr) => match addr {
                Some(addr) => {
                    let block = self.blocks.read(*addr as u64);

                    self.read_cluster(decode!(&block), range, buf)
                },
                None => Err(Error::ExpectedFile),
            },
            None => Err(Error::InvalidPath),
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

    let mut path: [u8; 56] = [0; 56];

    // /home/proxin/test.txt
    path[0..21].copy_from_slice(&[47, 104, 111, 109, 101, 47, 112, 114, 111, 120, 105, 110, 47, 116, 101, 115, 116, 46, 116, 120, 116]);

    let result = FILE_SYSTEM.lock().get_mut().unwrap().read(0..15, path, &mut [0]);

    log!("result: {:?}", result);
}


