use crate::log;

use alloc::vec::Vec;

use elf::section::SectionHeader;
use elf::segment::ProgramHeader;
use elf::endian::AnyEndian;
use elf::ElfBytes;

use stdlib::error::Error;


pub struct Program {
    program: Vec<u8>,
}

impl Program {
}

pub enum Data {
    Segment(ProgramHeader),
    Section(SectionHeader),
}

impl Data {
}

pub struct Loader<'a> {
    elf: ElfBytes<'a, AnyEndian>,
    data: Vec<Data>,
}

impl<'a> Loader<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Loader<'a>, Error> {
        let elf = ElfBytes::minimal_parse(bytes).map_err(|_| Error::InvalidElf)?;

        let segments = elf.segments()
            .map(|segments| segments.iter().map(|segment| Data::Segment(segment)).collect::<Vec<Data>>()).unwrap_or(Vec::new());

        let sections = elf.section_headers()
            .map(|headers| headers.iter().map(|header| Data::Section(header)).collect::<Vec<Data>>()).unwrap_or(Vec::new());

        Ok(Loader {
            elf,
            data: ,
        })
    }

    fn range(&self) {
    }

    fn load_segments(&self) -> Result<(), > {
        if let Some(segments) = self.elf.segments() {
            for segment in segments {
                let data = self.elf.segment_data(&segment).map_err(|_| Error::InvalidElf)?;

                log!("segment: {:?}", segment);
            }
        }
    }

    fn load_sections(&self) {
        if let Some(headers) = self.elf.section_headers() {
            for header in headers {
                let data = self.elf.section_data(&header).map_err(|_| Error::InvalidElf)?;

                header.

                log!("header: {:?}", header);
            }
        }
    }

    pub fn load_memory(&mut self) {
        // TODO: we need to load both segments and headers into memory
    }
}


