#![feature(let_chains)]
#![feature(string_remove_matches)]
#![feature(duration_millis_float)]

use std::{io::Write, sync::{atomic::AtomicBool, mpsc::channel, Arc}};

use arrayvec::ArrayVec;
use colored::Colorize;
use utils::{board::Board, consts::PIECE_INDICES, piece::Tile, piece_move::Move};

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

    // let mut guesses = ArrayVec::new();
    // let mut moves = ArrayVec::new();

    // for i in 0..218 {
    //     guesses.push(rand::random::<i32>());

    //     // rand number from 0-63
    //     let rand = rand::random::<u8>() % 64;
    //     let (rank, file) = (rand / 8, rand % 8);

    //     let rand2 = rand::random::<u8>() % 64;
    //     let (rank2, file2) = (rand2 / 8, rand2 % 8);

    //     moves.push(Move::new(Tile::new(rank, file).unwrap(), Tile::new(rank2, file2).unwrap(), utils::piece_move::MoveFlags::None));
    // }

    // let mut guesses_clone_1 = guesses.clone();
    // let mut moves_clone_1 = moves.clone();

    // let mut guesses_clone_2 = guesses.clone();
    // let mut moves_clone_2 = moves.clone();
    
    // naive(&mut guesses_clone_1, &mut moves_clone_1);
    // smart(&mut guesses_clone_2, &mut moves_clone_2);

    // assert_eq!(guesses_clone_1, guesses_clone_2);
    // assert_eq!(moves_clone_1, moves_clone_2);

    // let mut sum_naive = 0.0;
    // let mut sum_smart = 0.0;

    // for i in 0..1_000_000 {
    //     let time = std::time::Instant::now();
    //     naive(&mut guesses.clone(), &mut moves.clone());
    //     sum_naive += time.elapsed().as_millis_f64();

    //     let time = std::time::Instant::now();
    //     smart(&mut guesses.clone(), &mut moves.clone());
    //     sum_smart += time.elapsed().as_millis_f64();
    // }

    // dbg!(sum_naive);
    // dbg!(sum_smart);

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