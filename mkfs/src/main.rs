use std::os::unix::fs::FileExt;
use std::fs::{self, File, Metadata};
use std::mem;


#[repr(C)]
pub struct Header {
    magic: u32,
    entries: u32,
}

#[repr(C)]
pub struct Cluster {
    next: Option<usize>,
    len: u32,

    data: [u8; 512 - mem::size_of::<Option<usize>>() - mem::size_of::<u32>()],
}

// TODO: it would be nice if we could make sure that the size of this is a multiple of a 8 so that
// it doesnt cut across sectors
#[repr(C)]
pub struct DirEntry {
    name: [u8; 56],
    addr: Option<u32>,
}

pub struct Block {
    file: File,
}

impl Block {
    pub fn new(path: &str) -> Result<Block, Box<dyn std::error::Error>> {
        let file = File::create(path)?;

        file.set_len(65536)?;

        Ok(Block {
            file,
        })
    }

    pub fn write(&mut self, sector: u64, buf: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
        self.file.write_at(buf, sector * 512)
            .map_err(|err| err.into())
    }

    pub fn read(&mut self, sector: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf: [u8; 512] = [0; 512];

        self.file.read_at(&mut buf, sector * 512)
            .map_err(|err| err.into())
            .map(|read| buf[..read].to_vec())
    }
}

pub struct Mkfs {
    block: Block,
    entries: Vec<DirEntry>,
}

impl Mkfs {
    pub fn new(path: &str) -> Result<Mkfs, Box<dyn std::error::Error>> {
        Ok(Mkfs {
            block: Block::new(path)?,
            entries: Vec::new(),
        })
    }

    fn handle_entry(&mut self, path: &str, metadata: Metadata) -> Result<(), Box<dyn std::error::Error>> {
        println!("append_entry: path={:?}, metadata: {:?}", path, metadata);

        let mut name = path.as_bytes().to_vec();

        name.resize(56, 0);

        if metadata.is_dir() {
            self.entries.push(DirEntry {
                name: name.try_into().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to convert"))?,
                addr: None,
            });

            self.make(path)
        } else {
            // TODO: here we will have to allocate clusters for the file

            self.entries.push(DirEntry {
                name: name.try_into().map_err(|_| Into::<Box<dyn std::error::Error>>::into("failed to convert"))?,
                addr: None,
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

    fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", mem::size_of::<DirEntry>());

    let mut mkfs = Mkfs::new("hdd.dsk")?;

    mkfs.make("../user")?;

    mkfs.flush()
}


