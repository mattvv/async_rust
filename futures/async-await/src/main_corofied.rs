use std::time::Instant;

mod http;
mod future;

use future::*;
use crate::http::Http;

fn get_path(i: usize) -> String {
    format!("/{}/HelloWorld{i}", i * 1000)
}



fn main() {
    let start = Instant::now();
    let mut future = async_main();

    loop {
        match future.poll() {
            PollState::Ready(()) => ()),
            PollState::NotReady => break,
    }

    println!("\nELAPSED TIME: {}", start.elapsed().as_secs_f32());
}


// =================================
// We rewrite this:
// =================================
    
// coroutine fn async_main() {
//     println!("Program starting");
// 
//     let tx = Http::get(get_path(1));
//     println!("{txt}");
//     let txt = Http::get(get_path(2));
//     println!("{txt}");
//     let txt = Http::get(get_path(3));
//     println!("{txt}");
//     let txt = Http::get(get_path(4));
//     println!("{txt}");

// }

// =================================
// Into this:
// =================================

fn async_main() -> impl Future<Output=String> {
    Coroutine0::new()
}
        
enum State0 {
    Start,
    Resolved,
}

struct Coroutine0 {
    state: State0,
}

impl Coroutine0 {
    fn new() -> Self {
        Self { state: State0::Start }
    }
}


impl Future for Coroutine0 {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
        match self.state {
                State0::Start => {
                    // ---- Code you actually wrote ----
                    println!("Program starting");

    let tx = Http::get(get_path(1));
    println!("{txt}");
    let txt = Http::get(get_path(2));
    println!("{txt}");
    let txt = Http::get(get_path(3));
    println!("{txt}");
    let txt = Http::get(get_path(4));
    println!("{txt}");

                    // ---------------------------------
                    self.state = State0::Resolved;
                    break PollState::Ready(String::new());
                }

                State0::Resolved => panic!("Polled a resolved future")
            }
        }
    }
}
