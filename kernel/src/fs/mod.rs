mod error;
mod vfs;

use error::Error;

use crate::drivers::virtio::blk::{VirtioBlk, Mode};
use crate::log;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::vec;

use core::cell::OnceCell;
use core::ops::Range;
use core::iter;
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
    data: [u8; 500],
}

impl Cluster {
    pub fn new(next: Option<u32>, bytes: &[u8], len: u32) -> Cluster {
        let mut data: [u8; 500] = [0; 500];

        if bytes.len() > 0 {
            data[0..bytes.len()].copy_from_slice(&bytes);
        }

        Cluster {
            next,
            len,
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
    zones: Vec<bool>,
}

impl ZMap {
    pub fn new(len: usize) -> ZMap {
        // | header (1 sector) | directory table (256 sectors) | clusters |
        let mut zones = vec![true; 257];

        zones.extend(vec![false; len]);

        ZMap {
            zones,
        }
    }

    pub fn set(&mut self, zone: u32, status: bool) {
        self.zones[zone as usize] = status;
    }

    pub fn alloc(&mut self) -> Result<u32, Error> {
        match self.zones.iter().enumerate().find(|(_, zone)| **zone == false) {
            Some((index, _)) => {
                self.zones[index] = true;

                Ok(index as u32)
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
        let zmap = ZMap::new(driver.capacity as usize / 512);

        let mut fs = Fs {
            zmap,
            blocks: Blocks::new(driver),
            cache: BTreeMap::new(),
        };

        fs.init();

        fs
    }

    #[inline]
    fn cache_cluster(&mut self, mut cluster: Cluster) {
        while let Some(addr) = cluster.next {
            self.zmap.set(addr, true);

            log!("zone: {}", addr);

            cluster = decode!(&self.blocks.read(addr as u64));
        }
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
                    self.zmap.set(zone, true);

                    log!("zone: {}", zone);

                    let cluster = decode!(&self.blocks.read(zone as u64));

                    self.cache_cluster(cluster);
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
                buf.extend(&cluster.data[..cluster.len as usize]);
            }

            count += cluster.len;

            cluster = decode!(&self.blocks.read(cluster.next.ok_or(Error::OutOfBounds)? as u64));
        }
    }

    fn write_cluster(&mut self, mut cluster: Cluster, mut cluster_addr: u32, offset: u32, data: &[u8]) -> Result<(), Error> {
        let mut count: u32 = 0;

        loop {
            if offset >= count && offset < count + cluster.len {
                let hook = self.zmap.alloc()?;

                let split = Cluster::new(cluster.next, &cluster.data[offset as usize - count as usize..], cluster.len - (offset - count).min(cluster.len));

                unsafe {
                    self.blocks.write(hook as u64, mem::transmute_copy(&split));
                }

                let chain = self.chain_cluster(data, hook)?;

                cluster = Cluster::new(Some(chain), &cluster.data[..offset as usize - count as usize], offset - count);

                unsafe {
                    self.blocks.write(cluster_addr as u64, mem::transmute_copy(&cluster));
                }

                return Ok(());
            }

            count += cluster.len;

            match cluster.next {
                Some(next) => {
                    cluster_addr = next;

                    cluster = decode!(&self.blocks.read(cluster_addr as u64));
                },
                None => {
                    return Err(Error::OutOfBounds);
                },
            }
        }
    }

    fn chain_cluster(&mut self, data: &[u8], hook: u32) -> Result<u32, Error> {
        let zones = iter::repeat_with(|| self.zmap.alloc()).take((data.len() / 500) + 1).collect::<Vec<Result<u32, Error>>>();

        for (index, bytes) in data.chunks(500).enumerate() {
            let next = (index < zones.len() - 1).then(|| zones[index + 1].expect("internal error")).or_else(|| Some(hook));
            let cluster = Cluster::new(next, bytes, bytes.len() as u32);

            unsafe {
                self.blocks.write(zones[index]? as u64, mem::transmute_copy(&cluster));
            }
        }

        Ok(zones[0]?)
    }

    fn defrag(&mut self, mut cluster: Cluster) {
        // TODO: implement a defragmentation algorithm to prevent excessive fragmentation
    }

    fn dump_cluster(&mut self, mut cluster: Cluster) {
        loop {
            log!("cluster: {:?}", cluster);

            match cluster.next {
                Some(next) => {
                    cluster = decode!(&self.blocks.read(next as u64));
                },
                None => {
                    break;
                },
            }
        }
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

            fs.write_cluster(decode!(&block), addr, 40, &[104, 111, 109, 101]).unwrap();

            let block = fs.blocks.read(addr as u64);

            fs.dump_cluster(decode!(&block));

            let bytes = fs.read_cluster(decode!(&block), 38..100).unwrap();

            log!("bytes: {:?}", alloc::string::String::from_utf8_lossy(&bytes));
        },
        Err(err) => {
            log!("failed to query: {:?}", err);
        },
    }
}


