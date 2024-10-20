use core::arch::asm;

const SSIZE: isize = 48;

#[derive(Debug, Default)]
#[repr(C)]
struct THREADCONTEXT {
    rsp: u64,
}

fn main() {
    println!("Hello, world!");
}
