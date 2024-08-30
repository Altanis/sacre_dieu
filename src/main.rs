#![feature(let_chains)]
#![feature(string_remove_matches)]
#![feature(duration_millis_float)]
#![allow(clippy::identity_op)]
#![allow(clippy::wrong_self_convention)]
#![allow(unused_assignments)]

use std::sync::{atomic::AtomicBool, mpsc::channel, Arc};

mod engine;
mod utils;
mod uci;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = stop_signal.clone();

    let (sender, receiver) = channel();
    let _ = std::thread::spawn(move || uci::handle_board(receiver, stop_signal));

    let mut buffer = String::new();
    while std::io::stdin().read_line(&mut buffer).unwrap() > 0 {
        uci::handle_command(buffer.trim(), sender.clone(), stop_signal_clone.clone());
        buffer.clear();
    }
}