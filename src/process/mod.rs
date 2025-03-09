use crate::exception::TrapFrame;

use alloc::string::String;
use alloc::vec::Vec;

use spin::Lazy;


pub static PROCESSES: Lazy<Processes> = Lazy::new(|| Processes::new());


#[derive(Debug, Clone, Copy)]
pub enum State {
    Running,
    Sleeping,
    Zombie,
}

#[derive(Debug, Clone)]
pub struct Context {
    frame: TrapFrame,
    epc: u64,
}

impl Context {
    pub fn new(frame: TrapFrame, epc: u64) -> Context {
        Context {
            frame,
            epc,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Process {
    name: String,
    state: State,
    context: Context,
}

impl Process {
    pub fn new(name: String, state: State, context: Context) -> Process {
        Process {
            name,
            state,
            context,
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


