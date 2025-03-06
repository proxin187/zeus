use alloc::vec::Vec;

use spin::Lazy;


pub static PROCESSES: Lazy<Processes> = Lazy::new(|| Processes::new());


#[derive(Debug, Clone, Copy)]
pub struct Process {
    sp: u64,
}

impl Process {
    pub fn new(sp: u64) -> Process {
        Process {
            sp,
        }
    }
}

pub struct Processes {
    processes: Vec<Process>,
}

impl Processes {
    pub fn new() -> Processes {
        Processes {
            processes: Vec::new(),
        }
    }

    pub fn spawn(&mut self, entry: *const u8) {
    }
}


