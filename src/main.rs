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

    // let board = Board::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    // let piece_move = Move::from_uci("f3h3");

    // let e = board.static_eval_exchange(piece_move, 0);
    // if e {
    //     panic!("die")
    // }

    /// The PSQT, where each tuple represents opening/endgame evaluations.
    let mut psqts: Vec<[(i8, i8); 64]> = vec![
        // Pawns
        [
            (0,  0), (0,  0), (0,  0), (0,  0), (0,  0), (0,  0), (0,  0), (0,  0),
            (50, 80), (50, 80), (50, 80), (50, 80), (50, 80), (50, 80), (50, 80), (50, 80),
            (10, 50), (10, 50), (20, 50), (30, 30), (30, 30), (20, 50), (10, 50), (10, 50),
            ( 5, 30), ( 5, 30), (10, 30), (25, 20), (25, 20), (10, 30), ( 5, 30), ( 5, 30),
            ( 0, 20), ( 0, 20), ( 0, 20), (20, 20), (20, 20), ( 0, 20), ( 0, 20), ( 0, 20),
            ( 5, 10), (-5, 10), (-10, 10), ( 0, 10), ( 0, 10), (-10, 10), (-5, 10), ( 5, 10),
            ( 5, 10), (10, 10), (10, 10), (-20,  0), (-20,  0), (10, 10), (10, 10), ( 5, 10),
            ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0),
        ],
        // Knights
        [
            (-50, -50), (-40, -40), (-30, -30), (-30, -30), (-30, -30), (-30, -30), (-40, -40), (-50, -50),
            (-40, -40), (-20, -20), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), (-20, -20), (-40, -40),
            (-30, -30), ( 0,  0), (10, 10), (15, 15), (15, 15), (10, 10), ( 0,  0), (-30, -30),
            (-30, -30), ( 5,  5), (15, 15), (20, 20), (20, 20), (15, 15), ( 5,  5), (-30, -30),
            (-30, -30), ( 0,  0), (15, 15), (20, 20), (20, 20), (15, 15), ( 0,  0), (-30, -30),
            (-30, -30), ( 5,  5), (10, 10), (15, 15), (15, 15), (10, 10), ( 5,  5), (-30, -30),
            (-40, -40), (-20, -20), ( 0,  0), ( 5,  5), ( 5,  5), ( 0,  0), (-20, -20), (-40, -40),
            (-50, -50), (-40, -40), (-30, -30), (-30, -30), (-30, -30), (-30, -30), (-40, -40), (-50, -50),
        ],
        // Bishops
        [
            (-20, -20), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-20, -20),
            (-10, -10), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), (-10, -10),
            (-10, -10), ( 0,  5), ( 5, 10), (10, 10), (10, 10), ( 5, 10), ( 0,  5), (-10, -10),
            (-10, -10), ( 5,  5), ( 5, 10), (10, 10), (10, 10), ( 5, 10), ( 5,  5), (-10, -10),
            (-10, -10), ( 0, 10), (10, 10), (10, 10), (10, 10), (10, 10), ( 0, 10), (-10, -10),
            (-10, -10), (10, 10), (10, 10), (10, 10), (10, 10), (10, 10), (10, 10), (-10, -10),
            (-10, -10), ( 5,  5), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 5,  5), (-10, -10),
            (-20, -20), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-10, -10), (-20, -20),
        ],
        // Rooks
        [
            ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0),
            ( 5,  5), (10, 10), (10, 10), (10, 10), (10, 10), (10, 10), (10, 10), ( 5,  5),
            (-5, -5), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), (-5, -5),
            (-5, -5), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), (-5, -5),
            (-5, -5), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), (-5, -5),
            (-5, -5), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), (-5, -5),
            (-5, -5), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), (-5, -5),
            ( 0,  0), ( 0,  0), ( 0,  0), ( 5,  5), ( 5,  5), ( 0,  0), ( 0,  0), ( 0,  0),
        ],
        // Queens
        [
            (-20, -20), (-10, -10), (-10, -10), (-5, -5), (-5, -5), (-10, -10), (-10, -10), (-20, -20),
            (-10, -10), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), (-10, -10),
            (-10, -10), ( 0,  5), ( 5,  5), ( 5,  5), ( 5,  5), ( 5,  5), ( 0,  0), (-10, -10),
            ( -5,  -5), ( 0,  5), ( 5,  5), ( 5,  5), ( 5,  5), ( 5,  5), ( 0,  0), ( -5,  -5),
            (  0,  -5), ( 0,  5), ( 5,  5), ( 5,  5), ( 5,  5), ( 5,  5), ( 0,  0), ( -5,  -5),
            (-10, -10), ( 5,  5), ( 5,  5), ( 5,  5), ( 5,  5), ( 5,  5), ( 0,  0), (-10, -10),
            (-10, -10), ( 0,  5), ( 0,  5), ( 0,  0), ( 0,  0), ( 0,  0), ( 0,  0), (-10, -10),
            (-20, -20), (-10, -10), (-10, -10), (-5, -5), (-5, -5), (-10, -10), (-10, -10), (-20, -20),
        ],
        // Kings
        [
            (-80, -20), (-70, -10), (-70, -10), (-70, -10), (-70, -10), (-70, -10), (-70, -10), (-80, -20),
            (-60, - 5), (-60,  0), (-60,  5), (-60,  5), (-60,  5), (-60,  5), (-60,  0), (-60, - 5),
            (-40, -10), (-50, 20), (-50, 30), (-60, 30), (-60, 30), (-50, 30), (-50, 20), (-40, -10),
            (-30, -15), (-40, 35), (-40, 45), (-50, 45), (-50, 45), (-40, 35), (-40, 30), (-30, -15),
            (-20, -20), (-30, 30), (-30, 40), (-40, 40), (-40, 40), (-30, 30), (-30, -15), (-20, -20),
            (-10, -25), (-20, 20), (-20, 25), (-20, 25), (-20, 25), (-20, 20), (-20, -20), (-10, -25),
            ( 20, -30), ( 20, -25), ( -5,  0), ( -5,  0), ( -5,  0), ( -5,  0), (-25, -30), (-30, -50),
            ( 20, -50), ( 30, -30), ( 10, -30), (  0, -30), (  0, -30), ( 10, -30), ( 30, -30), ( 20, -50),
        ]
    ];

    for i in 0..psqts.len() {
        let psqt: [(i8, i8); 64] = psqts[i];
        let mut psqt_flipped: [(i8, i8); 64] = [(0, 0); 64];

        // flip the ranks
        for rank in 0..8 {
            for file in 0..8 {
                let idx = rank * 8 + file;
                let flipped_idx = (7 - rank) * 8 + file;
                psqt_flipped[flipped_idx] = psqt[idx];
            }
        }

        psqts.push(psqt_flipped);
    }

    // for arr in psqts.iter() {
    //     println!("[\n");
    //     for row in 0..8 {
    //         print!("\t");
    //         for col in 0..8 {
    //             let index = row * 8 + col;
    //             print!("({}, {}), ", arr[index].0, arr[index].1);
    //         }

    //         println!();
    //     }

    //     println!("],");
    // }

    let board = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let board2 = Board::new("4k3/8/8/8/8/8/8/4K3 w - - 0 1");

    // dbg!(board.endgame_weight());
    // dbg!(board2.endgame_weight());

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