mod error;
mod vfs;

use error::Error;

use crate::drivers::virtio::blk::{VirtioBlk, Mode};
use crate::log;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use core::cell::OnceCell;
use core::ops::Range;
use core::iter;
use core::mem;
use core::ptr;

use bitvec::vec::BitVec;
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

impl Cluster {
    pub fn new(next: Option<u32>, bytes: &[u8]) -> Cluster {
        let mut data: [u8; 492] = [0; 492];

        data[0..bytes.len()].copy_from_slice(&bytes);

        Cluster {
            next,
            len: bytes.len() as u32,
            data,
        }
    }
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
    pub fn new(driver: VirtioBlk) -> Blocks {
        Blocks {
            driver,
        }
    }

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

pub struct ZMap {
    zones: BitVec,
}

impl ZMap {
    pub fn new(len: usize) -> ZMap {
        // | header (1 sector) | directory table (256 sectors) | clusters |
        let mut zones = BitVec::repeat(true, 257);

        let clusters: BitVec = BitVec::repeat(false, len - 257);

        zones.extend(clusters);

        ZMap {
            zones,
        }
    }

    pub fn set(&mut self, zone: usize, status: bool) {
        self.zones.set(zone, status);
    }

    pub fn alloc(&mut self) -> Result<u32, Error> {
        match self.zones.first_zero() {
            Some(zone) => {
                self.zones.set(zone, true);

                Ok(zone as u32)
            },
            None => Err(Error::LimitedSpace),
        }
    }
}

pub struct Fs {
    zmap: ZMap,
    blocks: Blocks,
    cache: BTreeMap<[u8; 56], Option<u32>>,
}

impl Fs {
    pub fn new(driver: VirtioBlk) -> Fs {
        let mut fs = Fs {
            zmap: ZMap::new(driver.capacity as usize / 512),
            blocks: Blocks::new(driver),
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

                if let Some(zone) = entry.addr {
                    self.zmap.set(zone as usize, true);
                }
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

    fn read_cluster(&mut self, mut cluster: Cluster, range: Range<u32>) -> Result<Vec<u8>, Error> {
        let mut count: u32 = 0;
        let mut buf: Vec<u8> = Vec::new();

        loop {
            // TODO: this is a quick and easy solution, but im sure that you can find a generic function
            // that applies to all cases
            if range.start >= count && range.start < count + cluster.len {
                if range.end >= count && range.end < count + cluster.len {
                    buf.extend(&cluster.data[range.start as usize..range.end as usize]);

                    return Ok(buf);
                } else {
                    buf.extend(&cluster.data[range.start as usize - count as usize..cluster.len as usize]);
                }
            } else if range.end >= count && range.end < count + cluster.len {
                buf.extend(&cluster.data[..range.end as usize - count as usize]);

                return Ok(buf);
            } else if range.start < count && range.end >= count {
                buf.extend(&cluster.data);
            }

            count += cluster.len;

            cluster = decode!(&self.blocks.read(cluster.next.ok_or(Error::OutOfBounds)? as u64));
        }
    }

    fn write_cluster(&mut self, mut cluster: Cluster, offset: u32, data: &[u8]) -> Result<(), Error> {
        let mut count: u32 = 0;

        loop {
            if offset >= count && offset < count + cluster.len {
                // TODO: here we will have to split the cluster to insert the new cluster chain in
                // between

                // let addr = self.chain_cluster(data, hook)?;
            }

            count += cluster.len;

            cluster = decode!(&self.blocks.read(cluster.next.ok_or(Error::OutOfBounds)? as u64));
        }
    }

    fn chain_cluster(&mut self, data: &[u8], hook: u32) -> Result<u32, Error> {
        let zones = iter::repeat_with(|| self.zmap.alloc()).take(data.len() / 492).collect::<Vec<Result<u32, Error>>>();

        for (index, bytes) in data.chunks(492).enumerate() {
            let next = (index < zones.len() - 1).then_some(zones[index + 1]?).or_else(|| Some(hook));
            let cluster = Cluster::new(next, data);

            unsafe {
                self.blocks.write(zones[index]? as u64, mem::transmute_copy(&cluster));
            }
        }

        Ok(zones[0]?)
    }

    fn query(&self, path: [u8; 56]) -> Result<u32, Error> {
        match self.cache.get(&path) {
            Some(addr) => match addr {
                Some(addr) => Ok(*addr),
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

    let mut lock = FILE_SYSTEM.lock();

    let fs = lock.get_mut().unwrap();

    match fs.query(path) {
        Ok(addr) => {
            let block = fs.blocks.read(addr as u64);

            let bytes = fs.read_cluster(decode!(&block), 38..100).unwrap();

            log!("bytes: {:?}", alloc::string::String::from_utf8_lossy(&bytes));

            fs.write_cluster(decode!(&block), 40, &[104, 111, 109, 101]).unwrap();

            let bytes = fs.read_cluster(decode!(&block), 38..100).unwrap();

            log!("bytes: {:?}", alloc::string::String::from_utf8_lossy(&bytes));
        },
        Err(err) => {
            log!("failed to query: {:?}", err);
        },
    }
}


