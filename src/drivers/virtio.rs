use crate::log;

use spin::{Lazy, Mutex};


pub static VIRTIO_BLK: Lazy<Mutex<VirtioBlk>> = Lazy::new(|| VirtioBlk::new());


pub enum Error {
    OutOfBounds,
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

    pad: [u8; 4096 - size_of::<Descriptor>() * 16 - size_of::<Available>()],

    used: Used,
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            descriptors: [Descriptor::default(); 16],
            avail: Available::default(),

            pad: [0; 4096 - size_of::<Descriptor>() * 16 - size_of::<Available>()],

            used: Used::default(),
        }
    }
}

#[repr(C)]
pub struct Request {
    _type: u32,
    reserved: u32,
    sector: u64,
    data: [u8; 512],
    status: u8,
}

impl Request {
    pub fn new() -> Request {
        Request {
            _type: 0,
            reserved: 0,
            sector: 0,
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

pub struct VirtioBlk {
    addr: u64,
    capacity: u64,
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

    fn read_blk(&mut self, sector: u64) -> Result<(), Error> {
        if sector >= self.capacity / 512 {
            Err(Error::OutOfBounds)
        } else {
            self.req.sector = sector;
            self.req._type = 0;

            Ok(())
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


