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
    current: usize,
}

// State s athread can be in
#[derive(PartialEq, Eq, Debug)]
enum State {
    Available, // Thread available and ready to be assigned a task
    Running,   // Thread is running
    Ready,     // Ready to move forward and resume execution
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
    rbp: u64,
}

impl Thread {
    fn new() -> Self {
        Thread {
            stack: vec![0_u8: DEFAULT_STACK_SIZE], //Allocate new stack on thread creation (not most efficient)
            ctx: ThreadContext::default(),
            state: State::Available,
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        //initialize a new base thread
        let base_thread = Thread {
            stack: vec![0_u8: DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
        };

        //we now ahve one thread on the runtime scheduler
        let mut threads = vec![base_thread];

        //instantiate the rest of the threads
        let mut available_threads: Vec<Thread> = (1..MAX_THREADS).map(|_| Thread::new()).collect();
        threads.append(&mut available_threads);

        Runtime {
            threads,
            current: 0, //runtime thread
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

    #[inline(never)]
    fn t_yield(&mut self) {
        let mut pos = self.current;

        // go through any threads that are available and ready to run
        while self.threads[pos].state != State::Available {
            pos += 1;
            if pos == self.threads.len() {
                pos = 0;
            }
            if pos == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        //move to next thread
        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;

        unsafe {
            //swap threads
            let old: *mut ThreadContext = &mut self.threads[old_pos].ctx;
            let new: *const ThreadContext = &self.threads[pos].ctx;

            //copy thread context to the system V registers (rdi is first arg and rsi second)
            asm!("call switch", in("rdi") old, in("rsi") new, clobber_abi("C"));
        }

        //prevent copmiler from optimizing our code away.
        self.threads.len() > 0
    }

    pub fn spawn(&mut self, f: fn()) {
        //find the first available thread
        let available = self
            .threads
            .iter_mut()
            .find(|t| t.state == State::Available)
            .expect("No available thread");

        let size = available.stack.len();

        unsafe {
            // set up the stack for SystemV
            let s_ptr = available.stack.as_mut_ptr().offset(size as isize);
            //make sure memory is 16 byte aligned
            let s_ptr = (s_ptr as usize & !15) as *mut u8;
            //write address to our guard function that's called when our thread finishes
            std::ptr::write(s_ptr.offset(-16) as *mut u64, guard as u64);
            //write address to our skip function that's called when our thread - used for handling gap whenw e return from f so guard can be called on 16 byte boundary
            //because guard needs to be 16 byte aligned for the ABI requirements
            std::ptr::write(s_ptr.offset(-24) as *mut u64, skip as u64);
            // our function we are going to run
            std::ptr::write(s_ptr.offset(-32) as *mut u64, f as u64);
            // set rsp to stack pointer of this function.
            available.ctx.rsp = s_ptr.offset(-32) as u64;
        }

        available.state = State::Ready;
    }

    // called when return from f so we can call guard on 16 byte boundary
    fn guard() {
        unsafe {
            let r_ptr = RUNTIME as *const Runtime;
            (*rt_ptr).t_return();
        }
    }

    #[naked]
    unsafe extern "C" fn skip() {
        // ret pops off the next value from the stack and jump to whatever instructions that address points to
        // which will be the guard. (but of course 16 byte aligned)
        asm!("ret", options(noreturn)))
    }

    // lets us call yield from anywhere
    // super unsafe do not do this at home kids.
    pub fn yield_thread() {
        unsafe {
            let r_ptr = RUNTIME as *const Runtime;
            (*r_ptr).t_yield();
        }
    }

    #[naked]
    #[no_mangle]
    unsafe extern "C" fn switch() {
        asm!(
            "mov [rdi + 0x00], rsp",
            "mov [rdi + 0x08], r15",
            "mov [rdi + 0x10], r14",
            "mov [rdi + 0x18], r13",
            "mov [rdi + 0x20], r12",
            "mov [rdi + 0x28], rbx",
            "mov [rdi + 0x30], rbp",
            "mov rsp, [rsi + 0x00]",
            "mov r15, [rsi + 0x08]",
            "mov r14, [rsi + 0x10]",
            "mov r13, [rsi + 0x18]",
            "mov r12, [rsi + 0x20]",
            "mov rbx, [rsi + 0x28]",
            "mov rbp, [rsi + 0x30]",
            "ret", options(noreturn)
        )
    }
}

fn main() {
    let mut runtime = Runtime::new();
    runtime.init();
    runtime.spawn(|| {
        println!("THREAD 1 STARTING");
        let id = 1;
        for i in 0..10 {
            println!("thread: {} counter: {}", id, i);
            Runtime::yield_thread();
        }
    });
    runtime.spawn(|| {
        println!("THREAD 2 STARTING");
        let id = 2;
        for i in 0..15 {
            println!("thread: {} counter: {}", id, i);
            Runtime::yield_thread();
        }
    });
    runtime.run();
}
