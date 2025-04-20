use elf::endian::AnyEndian;
use elf::ElfBytes;

use stdlib::error::Error;


pub struct Loader<'a> {
    elf: ElfBytes<'a, AnyEndian>,
}

impl<'a> Loader<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Loader<'a>, Error> {
        match ElfBytes::minimal_parse(bytes) {
            Ok(elf) => Ok(Loader { elf }),
            Err(_) => Err(Error::InvalidElf),
        }
    }

    fn load_segments(&self) {
        if let Some(segments) = self.elf.segments() {
            for segment in segments {
            }
        }
    }

    fn load_sections(&self) {
        if let Some(headers) = self.elf.section_headers() {
            for header in headers {
            }
        }
    }

    pub fn load_memory(&mut self) {
        // TODO: we need to load both segments and headers into memory
    }
}


