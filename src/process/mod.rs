use crate::exception::TrapFrame;

use alloc::vec::Vec;

use spin::Lazy;


pub static PROCESSES: Lazy<Processes> = Lazy::new(|| Processes::new());


#[derive(Debug, Clone, Copy)]
pub enum State {
    Running,
    Sleeping,
}

#[derive(Debug, Clone, Copy)]
pub struct Process {
    state: State,
    frame: TrapFrame,
}

impl Process {
    pub fn new(state: State, frame: TrapFrame) -> Process {
        Process {
            state,
            frame,
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

    pub fn spawn(&mut self, entry: u64) {
    }
}


