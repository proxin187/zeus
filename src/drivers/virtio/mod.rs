mod blk;

use crate::log;

use blk::{VirtioBlk, Mode, Error};

use core::ops::RangeInclusive;

const MMIO_RANGE: RangeInclusive<u64> = 0x1000_1000..=0x1000_8000;


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

#[derive(Debug)]
pub struct Devices {
    pub block: Option<VirtioBlk>,
}

impl Devices {
    pub fn new() -> Devices {
        Devices {
            block: None,
        }
    }

    fn is_filled(&self) -> bool {
        self.block.is_some()
    }

    pub fn read_blk(&mut self, sector: u64) -> Result<[u8; 512], Error> {
        let buffer: [u8; 512] = [0; 512];

        match &mut self.block {
            Some(block) => {
                unsafe {
                    block.blk_op(Mode::Read, &buffer as *const [u8; 512] as *mut [u8; 512], sector).map(|()| buffer)
                }
            },
            None => {
                panic!("block device unavailable");
            },
        }
    }

    pub fn write_blk(&mut self, sector: u64, buffer: [u8; 512]) -> Result<(), Error> {
        match &mut self.block {
            Some(block) => {
                unsafe {
                    block.blk_op(Mode::Write, &buffer as *const [u8; 512] as *mut [u8; 512], sector)
                }
            },
            None => {
                panic!("block device unavailable");
            },
        }
    }
}

pub enum DeviceType {
    Reserved,
    NetworkCard,
    BlockDevice,
    GpuDevice,
    InputDevice,
    Unknown,
}

impl From<u32> for DeviceType {
    fn from(value: u32) -> DeviceType {
        match value {
            0 => DeviceType::Reserved,
            1 => DeviceType::NetworkCard,
            2 => DeviceType::BlockDevice,
            16 => DeviceType::GpuDevice,
            18 => DeviceType::InputDevice,
            _ => DeviceType::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Device {
    addr: u64,
}

impl Device {
    pub fn new(addr: u64) -> Device {
        Device {
            addr,
        }
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
}

pub fn probe() -> Devices {
    let mut devices = Devices::new();

    log!("probing virtio devices: {:x?}", MMIO_RANGE);

    for addr in MMIO_RANGE.step_by(0x1000) {
        let device = Device::new(addr);

        if device.virtio_read::<u32>(MmioOffset::Magic) == 0x74726976 {
            let id = device.virtio_read::<u32>(MmioOffset::Id);

            match DeviceType::from(id) {
                DeviceType::Reserved => {},
                DeviceType::NetworkCard => {
                    log!("{:x?}: network card ignored", device);
                },
                DeviceType::BlockDevice => {
                    log!("{:x?}: virtio-blk device found", device);

                    if devices.block.replace(unsafe { VirtioBlk::new(device) }).is_some() {
                        panic!("multiple virtio-blk devices not supported");
                    }
                },
                DeviceType::GpuDevice => {
                    log!("{:x?}: gpu device ignored", device);
                },
                DeviceType::InputDevice => {
                    log!("{:x?}: input device ignored", device);
                },
                DeviceType::Unknown => {
                    panic!("{:x?}: unknown virtio device id: {}", device, id);
                },
            }
        }
    }

    if devices.is_filled() {
        devices
    } else {
        panic!("failed to find required virtio devices");
    }
}


