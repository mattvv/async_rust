use core::arch::asm;

const SSIZE: isize = 48;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64,
}

fn hello() -> ! {
    println!("I LOVE WAKING UP ON A NEW STACK!");
    loop {}
}

unsafe fn gt_switch(new: *const ThreadContext) {
    asm!(
        "mov rsp, [{0} + 0x00]",
        "ret",
        in(reg) new,
    )
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0_u8; SSIZE as usize];

    unsafe {
        let stack_bottom = stack.as_mut_ptr().offset(SSIZE);

        // Round our memory address to nearest 16 byte aligned address
        let sb_aligned = (stack_bottom as usize & !15) as *mut u8;

        // Write the address of the function we want to call to the stack
        // Cat to a u64 to avoid a default u8, otherwise we will write to write only to position 32.
        std::ptr::write(sb_aligned.offset(-16) as *mut u64, hello as u64);

        ctx.rsp = sb_aligned.offset(-16) as u64;

        // Show stack
        for i in 0..SSIZE {
            println!("mem: {}, val: {}",  sb_aligned.offset(-i as isize) as usize, *sb_aligned.offset(-i as isize));
        }

        gt_switch(&mut ctx);
    }
}
