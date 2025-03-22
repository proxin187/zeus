use super::{MmioOffset, Device};

use crate::{memory, log};

use core::alloc::{GlobalAlloc, Layout};
use core::mem;


#[derive(Debug)]
pub enum Error {
    OutOfBounds,
    Failed {
        status: u8,
        sector: u64,
    },
}

pub enum Flags {
    Next = 1,
    Write = 2,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Descriptor {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
#[derive(Default)]
pub struct Available {
    flags: u16,
    index: u16,
    ring: [u16; 16],
}

#[repr(C)]
#[derive(Default)]
pub struct UsedElement {
    id: u32,
    len: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct Used {
    flags: u16,
    index: u16,
    ring: [UsedElement; 16],
}

#[repr(C)]
pub struct Queue {
    descriptors: [Descriptor; 16],
    avail: Available,

    // TODO: this assumes that the address of Queue is a multiple of 4096, maybe we should manually
    // allocate it to be this?
    pad: [u8; 4096 - size_of::<Descriptor>() * 16 - size_of::<Available>()],

    used: Used,
    last_used: u16,
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            descriptors: [Descriptor::default(); 16],
            avail: Available::default(),

            pad: [0; 4096 - size_of::<Descriptor>() * 16 - size_of::<Available>()],

            used: Used::default(),
            last_used: 0,
        }
    }
}

#[repr(C)]
pub struct Header {
    _type: u32,
    reserved: u32,
    sector: u64,
}

#[repr(C)]
pub struct Request {
    header: Header,
    data: [u8; 512],
    status: u8,
}

impl Request {
    pub fn new() -> Request {
        Request {
            header: Header {
                _type: 0,
                reserved: 0,
                sector: 0,
            },
            data: [0; 512],
            status: 0,
        }
    }
}

pub enum VirtioStatus {
    Ack = 1,
    Driver = 2,
    DriverOk = 4,
    FeatOk = 8,
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    Read,
    Write,
}

impl Mode {
    pub fn _type(&self) -> u32 {
        match self {
            Mode::Read => 0,
            Mode::Write => 1,
        }
    }
}

pub struct VirtioBlk {
    device: Device,
    capacity: u64,

    virtq: u64,
    req: Request,
}

impl core::fmt::Debug for VirtioBlk {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        f.write_str("VirtioBlk {\n")?;

        f.write_fmt(format_args!("    device: {:x?},\n", self.device))?;
        f.write_fmt(format_args!("    capacity: {:x?},\n", self.capacity))?;

        f.write_str("    virtq: ...,\n")?;
        f.write_str("    req: ...,\n")?;

        f.write_str("}\n")?;

        Ok(())
    }
}

impl VirtioBlk {
    pub unsafe fn new(device: Device) -> VirtioBlk {
        let virtq = memory::ALLOC.alloc(Layout::from_size_align(mem::size_of::<Queue>(), 4096).expect("invalid layout")) as *mut Queue;

        *virtq = Queue::new();

        let mut virtio_blk = VirtioBlk {
            device,
            capacity: 0,

            virtq: virtq as u64,
            req: Request::new(),
        };

        virtio_blk.init_virtblk();

        virtio_blk
    }

    unsafe fn notify(&mut self) {
        let virtq = self.virtq as *const Queue as *mut Queue;

        (*virtq).avail.ring[(*virtq).avail.index as usize % 16] = 0;

        (*virtq).avail.index = (*virtq).avail.index.wrapping_add(1);

        self.device.virtio_write(MmioOffset::QueueNotify, 0);

        (*virtq).last_used = (*virtq).last_used.wrapping_add(1);
    }

    pub unsafe fn blk_op(&mut self, mode: Mode, buf: *mut [u8; 512], sector: u64) -> Result<(), Error> {
        if sector >= self.capacity / 512 {
            Err(Error::OutOfBounds)
        } else {
            let virtq = self.virtq as *const Queue as *mut Queue;

            self.req.header.sector = sector;
            self.req.header._type = mode._type();

            if mode == Mode::Write {
                self.req.data = *buf;
            }

            (*virtq).descriptors[0] = Descriptor {
                addr: &self.req.header as *const Header as u64,
                len: core::mem::size_of::<Header>() as u32,
                flags: Flags::Next as u16,
                next: 1,
            };

            (*virtq).descriptors[1] = Descriptor {
                addr: &self.req.data as *const [u8; 512] as u64,
                len: core::mem::size_of::<[u8; 512]>() as u32,
                flags: Flags::Next as u16 | if mode == Mode::Write { 0 } else { Flags::Write as u16 },
                next: 2,
            };

            (*virtq).descriptors[2] = Descriptor {
                addr: &self.req.status as *const u8 as u64,
                len: core::mem::size_of::<u8>() as u32,
                flags: Flags::Write as u16,
                next: 0,
            };

            self.notify();

            while (*virtq).last_used != (&(*virtq).used.index as *const u16).read_volatile() {}

            match self.req.status {
                0 => {
                    if mode == Mode::Read {
                        *buf = self.req.data;
                    }

                    Ok(())
                },
                status => Err(Error::Failed {
                    status,
                    sector,
                }),
            }
        }
    }

    fn init_virtq(&self) {
        // select the queue 0
        self.device.virtio_write(MmioOffset::QueueSel, 0);

        // give the queue size to the device
        self.device.virtio_write(MmioOffset::QueueNum, 16);

        // give alignment to the device
        self.device.virtio_write(MmioOffset::QueueAlign, 0);

        // give the address of the queue to the device
        self.device.virtio_write(MmioOffset::QueuePfn, self.virtq as u32);
    }

    fn init_virtblk(&mut self) {
        if self.device.virtio_read::<u32>(MmioOffset::Version) != 1 {
            panic!("device isnt version 1: {:#x?}", self.device);
        }

        // reset the device
        self.device.virtio_write(MmioOffset::Status, 0);

        // acknowlegde the device
        self.device.virtio_mask(MmioOffset::Status, VirtioStatus::Ack as u32);

        // set driver bit of status
        self.device.virtio_mask(MmioOffset::Status, VirtioStatus::Driver as u32);

        // set features ok bit of status
        self.device.virtio_mask(MmioOffset::Status, VirtioStatus::FeatOk as u32);

        // initialize the virtqueues
        self.init_virtq();

        // set the driver ok bit of status and clear others
        self.device.virtio_write(MmioOffset::Status, VirtioStatus::DriverOk as u32);

        self.capacity = self.device.virtio_read::<u64>(MmioOffset::Config) * 512;
    }
}


