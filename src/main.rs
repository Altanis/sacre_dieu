#![feature(let_chains)]
#![feature(string_remove_matches)]
#![feature(duration_millis_float)]

use std::{io::Write, sync::{atomic::AtomicBool, mpsc::channel, Arc}};

use arrayvec::ArrayVec;
use colored::Colorize;
use utils::{board::Board, consts::{PIECE_INDICES, PIECE_SQUARE_TABLE}, piece::{Piece, PieceColor, Tile}, piece_move::Move};

mod engine;
mod utils;
mod uci;

// fn naive(guesses: &mut ArrayVec<i32, 218>, moves: &mut ArrayVec<Move, 218>) {
//     let mut combined: Vec<(_, _)> = guesses.iter().copied().zip(moves.iter().copied()).collect();
//     combined.sort_by_key(|&(score, _)| score);
//     for (i, (score, mov)) in combined.into_iter().enumerate() {
//         guesses[i] = score;
//         moves[i] = mov;
//     }
// }

// fn smart(guesses: &mut ArrayVec<i32, 218>, moves: &mut ArrayVec<Move, 218>) {
//     let mut indices: ArrayVec<usize, 218> = (0..guesses.len()).collect();
//     indices.sort_unstable_by_key(|&i| guesses[i]);

//     let temp_guesses = guesses.clone();
//     let temp_moves = moves.clone();

//     for (new_idx, &i) in indices.iter().enumerate() {
//         guesses[new_idx] = temp_guesses[i];
//         moves[new_idx] = temp_moves[i];
//     }
// }

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