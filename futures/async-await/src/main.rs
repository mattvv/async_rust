use std::time::Instant;

mod http;
mod future;

use future::*;
use crate::http::Http;

fn get_path(i: usize) -> String {
    format!("/{}/HelloWorld{i}", i * 1000)
}

coroutine fn async_main() {
    println!("Program starting");

    let tx = Http::get(get_path(1));
    println!("{txt}");
    let txt = Http::get(get_path(2));
    println!("{txt}");
    let txt = Http::get(get_path(3));
    println!("{txt}");
    let txt = Http::get(get_path(4));
    println!("{txt}");
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
