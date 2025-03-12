use crate::log;


static VIRTIO_BLK: VirtioBlk = VirtioBlk::new();


pub enum MmioOffset {
    Magic = 0x00,
    Version = 0x04,
    Id = 0x08,
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
}

impl VirtioBlk {
    pub const fn new() -> VirtioBlk {
        VirtioBlk {
            addr: 0x10001000,
        }
    }

    #[inline]
    fn virtio_read(&self, offset: MmioOffset) -> u64 {
        unsafe {
            ((self.addr + offset as u64) as *const u64 as *mut u64).read_volatile()
        }
    }

    fn init(&self) {
        // TODO: we get invalid drive here
        if self.virtio_read(MmioOffset::Magic) != 0x74726976 || self.virtio_read(MmioOffset::Version) != 1 || self.virtio_read(MmioOffset::Id) != 2 {
            panic!("invalid drive");
        }

        log!("valid drive");
    }
}

pub fn init() {
    VIRTIO_BLK.init();
}


