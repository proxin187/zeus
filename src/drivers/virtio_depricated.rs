// this file contains an old and broken virtio implementation, for now we are using a crate for
// virtio instead.

use crate::log;

use spin::{Lazy, Mutex};


pub static VIRTIO_BLK: Lazy<Mutex<VirtioBlk>> = Lazy::new(|| VirtioBlk::new());


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

#[derive(Debug, Clone, Copy)]
pub enum MmioOffset {
    Magic = 0x00,
    Version = 0x04,
    Id = 0x08,
    QueueSel = 0x30,
    QueueNumMax = 0x34,
    QueueNum = 0x38,
    QueueAlign = 0x3c,
    QueuePfn = 0x40,
    QueueNotify = 0x50,
    Status = 0x70,
    Config = 0x100,
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
    addr: u64,
    capacity: u64,

    // TODO: manually allocate so that its aligned with 4096
    virtq: Queue,
    req: Request,
}

impl VirtioBlk {
    pub fn new() -> Mutex<VirtioBlk> {
        let mut virtio_blk = VirtioBlk {
            addr: 0x10001000,
            capacity: 0,
            virtq: Queue::new(),
            req: Request::new(),
        };

        virtio_blk.init_virtblk();

        Mutex::new(virtio_blk)
    }

    #[inline]
    fn virtio_read<T>(&self, offset: MmioOffset) -> T {
        unsafe {
            ((self.addr + offset as u64) as *const T as *mut T).read_volatile()
        }
    }

    #[inline]
    fn virtio_write(&self, offset: MmioOffset, value: u32) {
        unsafe {
            ((self.addr + offset as u64) as *const u32 as *mut u32).write_volatile(value);
        }
    }

    #[inline]
    fn virtio_mask(&self, offset: MmioOffset, value: u32) {
        self.virtio_write(offset, self.virtio_read::<u32>(offset) | value);
    }

    fn notify(&mut self) {
        self.virtq.avail.ring[self.virtq.avail.index as usize % 16] = 0;

        self.virtq.avail.index = self.virtq.avail.index.wrapping_add(1);

        self.virtio_write(MmioOffset::QueueNotify, 0);

        self.virtq.last_used = self.virtq.last_used.wrapping_add(1);
    }

    fn blk_op(&mut self, mode: Mode, buf: *mut [u8; 512], sector: u64) -> Result<(), Error> {
        if sector >= self.capacity / 512 {
            Err(Error::OutOfBounds)
        } else {
            log!("blk_op: mode={:?}, buf={:?}, sector={}", mode, buf, sector);

            self.req.header.sector = sector;
            self.req.header._type = mode._type();

            if mode == Mode::Write {
                self.req.data = unsafe { *buf };
            }

            self.virtq.descriptors[0] = Descriptor {
                addr: &self.req.header as *const Header as u64,
                len: core::mem::size_of::<Header>() as u32,
                flags: Flags::Next as u16,
                next: 1,
            };

            self.virtq.descriptors[1] = Descriptor {
                addr: &self.req.data as *const [u8; 512] as u64,
                len: core::mem::size_of::<[u8; 512]>() as u32,
                flags: Flags::Next as u16 | Flags::Write as u16,
                next: 2,
            };

            self.virtq.descriptors[2] = Descriptor {
                addr: &self.req.status as *const u8 as u64,
                len: core::mem::size_of::<u8>() as u32,
                flags: Flags::Write as u16,
                next: 0,
            };

            self.notify();

            // TODO: it hangs here, it doesnt seem to recognize when the operation is done
            log!("waiting for read to be done");

            unsafe {
                while self.virtq.last_used != (&self.virtq.used.index as *const u16).read_volatile() {}
            }

            log!("read is done");

            match self.req.status {
                0 => {
                    if mode == Mode::Read {
                        unsafe {
                            *buf = self.req.data;
                        }
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
        self.virtio_write(MmioOffset::QueueSel, 0);

        // give the queue size to the device
        self.virtio_write(MmioOffset::QueueNum, 16);

        // give alignment to the device
        self.virtio_write(MmioOffset::QueueAlign, 0);

        // give the address of the queue to the device
        self.virtio_write(MmioOffset::QueuePfn, &self.virtq as *const Queue as u32);
    }

    fn init_virtblk(&mut self) {
        if self.virtio_read::<u32>(MmioOffset::Magic) != 0x74726976 || self.virtio_read::<u32>(MmioOffset::Version) != 1 || self.virtio_read::<u32>(MmioOffset::Id) != 2 {
            panic!("invalid drive: {:#x?}", self.addr);
        } else {
            log!("found drive: {:#x?}", self.addr);
        }

        // reset the device
        self.virtio_write(MmioOffset::Status, 0);

        // acknowlegde the device
        self.virtio_mask(MmioOffset::Status, VirtioStatus::Ack as u32);

        // set driver bit of status
        self.virtio_mask(MmioOffset::Status, VirtioStatus::Driver as u32);

        // set features ok bit of status
        self.virtio_mask(MmioOffset::Status, VirtioStatus::FeatOk as u32);

        // initialize the virtqueues
        self.init_virtq();

        // set the driver ok bit of status and clear others
        self.virtio_write(MmioOffset::Status, VirtioStatus::DriverOk as u32);

        self.capacity = self.virtio_read::<u64>(MmioOffset::Config) * 512;

        log!("virtio-blk capacity: {:#x?}", self.capacity);
    }
}

pub fn read(sector: u64) -> Result<[u8; 512], Error> {
    let buffer: [u8; 512] = [0; 512];

    log!("read: {}", sector);

    VIRTIO_BLK.lock()
        .blk_op(Mode::Read, &buffer as *const [u8; 512] as *mut [u8; 512], sector)
        .map(|()| buffer)
}

pub fn write(sector: u64, buffer: [u8; 512]) -> Result<(), Error> {
    VIRTIO_BLK.lock()
        .blk_op(Mode::Write, &buffer as *const [u8; 512] as *mut [u8; 512], sector)
}


