use std::os::unix::fs::FileExt;
use std::fs::{self, File, Metadata};
use std::slice;
use std::mem;


macro_rules! encode {
    ($value:expr) => {
        unsafe {
            slice::from_raw_parts($value as *const _ as *const u8, mem::size_of_val($value))
        }
    };
}

#[repr(C)]
pub struct Header {
    magic: u32,
    entries: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct Cluster {
    next: Option<u32>,
    len: u32,
    data: [u8; 500],
}

#[repr(C)]
#[derive(Debug)]
pub struct DirEntry {
    name: [u8; 56],
    addr: Option<u32>,
}

pub struct Mkfs<'a> {
    file: File,
    entries: Vec<DirEntry>,
    cluster: u32,
    dir: &'a str,
}

impl<'a> Mkfs<'a> {
    pub fn new(path: &str, dir: &'a str) -> Result<Mkfs<'a>, Box<dyn std::error::Error>> {
        let file = File::create(path)?;

        // this is 1 gigabyte
        file.set_len(1074000000)?;

        Ok(Mkfs {
            file,
            entries: Vec::new(),
            cluster: 256,
            dir,
        })
    }

    fn cluster(&mut self, absolute: &str) -> Result<u32, Box<dyn std::error::Error>> {
        let bytes = fs::read(absolute)?;
        let chunks = bytes.chunks(500);
        let total = chunks.len();

        let cluster = self.cluster;

        for (index, mut chunk) in chunks.map(|chunk| chunk.to_vec()).enumerate() {
            let len = chunk.len() as u32;

            chunk.resize(500, 0);

            let cluster = Cluster {
                next: (index + 1 < total).then(|| self.cluster + 1),
                len,
                data: chunk.try_into().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to convert"))?,
            };

            println!("cluster: cluster={:?}, addr={:?}", cluster, self.cluster as u64 * 512);

            self.file.write_at(encode!(&cluster), self.cluster as u64 * 512)?;

            self.cluster += 1;
        }

        Ok(cluster)
    }

    fn handle_entry(&mut self, absolute: &str, metadata: Metadata) -> Result<(), Box<dyn std::error::Error>> {
        let (_, relative) = absolute.split_at(self.dir.len());

        println!("append_entry: absolute={:?}, relative={:?}, metadata: {:?}", absolute, relative, metadata);

        let mut name = relative.as_bytes().to_vec();

        name.resize(56, 0);

        if metadata.is_dir() {
            self.entries.push(DirEntry {
                name: name.try_into().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to convert"))?,
                addr: None,
            });

            self.make(absolute)
        } else {
            let cluster = self.cluster(absolute)?;

            self.entries.push(DirEntry {
                name: name.try_into().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to convert"))?,
                addr: Some(cluster),
            });

            Ok(())
        }
    }

    fn make(&mut self, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    self.handle_entry(&path.to_string_lossy(), entry.metadata()?)?;
                },
                Err(err) => {
                    println!("failed to get entry: {:?}", err);
                },
            }
        }

        Ok(())
    }

    fn header(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let header = Header {
            magic: 0x5a455553,
            entries: self.entries.len() as u32,
        };

        self.file.write_at(encode!(&header), 0)?;

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (index, entry) in self.entries.iter().enumerate() {
            println!("flush: entry={:?}", entry);

            self.file.write_at(encode!(entry), 512 + (index as u64 * mem::size_of::<DirEntry>() as u64))?;
        }

        self.header()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut mkfs = Mkfs::new("../hdd.dsk", "../user")?;

    mkfs.make("../user")?;

    mkfs.flush()
}


