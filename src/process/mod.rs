use crate::exception::TrapFrame;
use crate::log;

use alloc::string::{ToString, String};
use alloc::vec::Vec;
use alloc::vec;

use spin::{Mutex, Lazy};


pub static PROCESSES: Lazy<Mutex<Processes>> = Lazy::new(|| Mutex::new(Processes::new()));


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Running,
    Runable,
    Sleeping,
    Zombie,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub frame: TrapFrame,
    pub epc: u64,
}

impl Context {
    pub fn new(frame: TrapFrame, epc: u64) -> Context {
        Context {
            frame,
            epc,
        }
    }
}

#[derive(Clone)]
pub struct Process {
    name: String,
    state: State,
    context: Context,
    stack: Vec<u8>,
}

impl core::fmt::Debug for Process {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        fmt.write_fmt(format_args!("Process {{ name: {:?}, state: {:?}, context: {:x?}}}", self.name, self.state, self.context))
    }
}

impl Process {
    pub fn new(name: String, state: State, context: Context, stack: Vec<u8>) -> Process {
        Process {
            name,
            state,
            context,
            stack,
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

    pub fn save_context(&mut self, context: Context) {
        if self.processes[0].state == State::Running {
            self.processes[0].context = context;

            self.processes[0].state = State::Runable;

            log!("processes addr: {:x?}", core::ptr::addr_of!(PROCESSES));
            log!("name addr: {:x?}", self.processes[0].name.as_ptr());

            // TODO: the kernel panics here
            // the issue might be related to memory allocation, maybe we get overlapping
            // allocations?
            //
            // nevermind, looks like we dont have permission to read the address where name is
            // stored?
            //
            // the address is being changed from 0x8425fffc to 0x80218f8800000000 and is therefore
            // invalid
            //
            // the data inside the static PROCESSES is being overwritten
            log!("process context saved: {:?}", self.processes[0]);
        }
    }

    pub fn next(&mut self) -> Context {
        self.processes.rotate_left(1);

        self.processes[0].state = State::Running;

        log!("next process loaded: {:?}", self.processes[0]);

        self.processes[0].context.clone()
    }
}

pub fn spawn(name: &str, entry: u64) {
    let mut process = Process::new(name.to_string(), State::Runable, Context::new(TrapFrame::new(), entry), vec![0; 128 * 1024]);

    // 0x8021f020
    log!("processes addr: {:x?}", core::ptr::addr_of!(PROCESSES));
    log!("name: {:?}, addr: {:x?}", process.name, process.name.as_ptr());

    process.context.frame.regs[1] = process.stack.as_ptr() as u64;

    log!("spawn: {:?}", process);

    PROCESSES.lock().processes.push(process);
}

pub fn schedule(context: Context) -> Context {
    let mut processes = PROCESSES.lock();

    processes.save_context(context);

    processes.next()
}


