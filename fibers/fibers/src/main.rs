// Example of implementing our own raw fibers on SystemV

// Use Naked functions since we're going to manually swap stacks in assembly.
// We don't want the OS adding prologue and epilogue between each stack.
#![feature(naked_functions)]
use std::arch::asm;

// 2MB Stack size.
const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;

const MAX_THREADS: usize = 4;

//global mutable variable, ew.
static mut RUNTIME: usize = 0;

pub struct Runtime {
    threads: Vec<Thread>,
    current: usize
}

// State s athread can be in
#[derive(PartialEq, Eq, Debug)]
enum State {
    Available, // Thread available and ready to be assigned a task
    Running, // Thread is running
    Ready // Ready to move forward and resume execution
}

// Hold all data for a thread.
struct Thread {
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
}

// Holds data for the registers that the CPU needs to resume execution on a stack
#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    //registers
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64
}

impl Thread {
    fn new() -> Self {
        Thread {
            stack: vec![0_u8: DEFAULT_STACK_SIZE], //Allocate new stack on thread creation (not most efficient)
            ctx: ThreadContext::default(),
            state: State::Available
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        //initialize a new base thread
        let base_thread = Thread {
            stack: vec![0_u8: DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running
        };

        //we now ahve one thread on the runtime scheduler
        let mut threads = vec![base_thread];

        //instantiate the rest of the threads
        let mut available_threads: Vec<Thread> = (1..MAX_THREADS).map(|_| Thread::new()).collect();
        threads.append(&mut available_threads);

        Runtime {
            threads,
            current: 0 //runtime thread
        }
    }

    pub fn init(&self) {
        //allow us to call yield from anywhere (simplest unsafe method for now, refactor later)
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.t_yield() {}
        std::process::exit(0);
    }

    // Called by stack when a thread is finished.
    fn t_return(&mut self) {
        if self.current != 0 {
            //calling thread is not base thread, so set to Available
            //So we're ready for next task
            self.threads[self.current].state = State::Available;

            //schedule new thread to be run
            self.t_yield();
        }
    }
}

fn main() {
    println!("Hello, world!");
}
