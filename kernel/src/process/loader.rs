use crate::log;

use alloc::vec::Vec;
use alloc::vec;

use core::ops::Range;

use elf::section::SectionHeader;
use elf::segment::ProgramHeader;
use elf::endian::AnyEndian;
use elf::ElfBytes;

use stdlib::error::Error;


// TODO: maybe this will have to be stored in the process structure?
pub struct Program {
    addr: u64,
    program: Vec<u8>,
}

impl Program {
    pub fn new(range: Range<u64>) -> Program {
        Program {
            addr: range.start,
            program: vec![0; range.end as usize - range.start as usize],
        }
    }

    pub fn insert(&mut self, bytes: &[u8]) {
        self.program.copy_from_slice(bytes);
    }
}

pub enum Data {
    Segment(ProgramHeader),
    Section(SectionHeader),
}

impl Data {
    pub fn addr(&self) -> u64 {
        match self {
            Data::Segment(header) => header.p_vaddr,
            Data::Section(section) => section.sh_addr,
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            Data::Segment(header) => header.p_memsz,
            Data::Section(section) => section.sh_size,
        }
    }

    pub fn bytes<'a>(&self, elf: ElfBytes<'a, AnyEndian>) -> Result<&[u8], Error> {
        match self {
            Data::Segment(header) => elf.segment_data(header),
            Data::Section(section) => section.sh_size,
        }
    }
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
            data: [segments, sections].into_iter().flatten().collect(),
        })
    }

    /*
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
    */

    pub fn load(&mut self) -> Result<Program, Error> {
        let min = self.data.iter().map(|data| data.addr()).min().unwrap_or(0);
        let max = self.data.iter().map(|data| data.addr() + data.size()).max().unwrap_or(0);

        let mut program = Program::new(min..max);

        for data in self.data.iter() {
        }

        Ok(program)
    }
}


