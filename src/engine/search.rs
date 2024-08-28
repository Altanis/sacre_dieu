use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::{Duration, Instant}};

use arrayvec::ArrayVec;

use crate::utils::{board::Board, consts::{BEST_EVAL, WORST_EVAL}, piece_move::{order_moves, Move}};
use super::eval;

pub struct Searcher {
    /// The time constraint of the search.
    pub time_limit: Duration,
    /// The timer associated with the search.
    pub timer: Instant,
    /// The maximum depth of the search.
    pub max_depth: usize,
    /// A boolean signalling when to stop a search.
    pub stop_signal: Arc<AtomicBool>,
    /// The number of nodes searched.
    pub nodes: usize,
    /// The past board positions, represented as zobrist hashes.
    pub past_boards: Vec<u64>
}

impl Searcher {
    /// Initializes a new searcher.
    pub fn new(time_limit: Duration, depth: usize, stop_signal: Arc<AtomicBool>) -> Self {
        Searcher {
            time_limit,
            timer: Instant::now(),
            max_depth: depth,
            stop_signal,
            nodes: 0,
            past_boards: Vec::new()
        }
    }

    /// Searches for a move with a time constraint.
    pub fn search_timed(&mut self, board: &Board) -> (i32, Option<Move>) {
        self.timer = std::time::Instant::now();
        let (mut eval, mut best_move): (i32, Option<Move>) = (0, None);

        let mut index = 0;
        self.max_depth = 0;

        while best_move.is_none() || (self.timer.elapsed() <= self.time_limit && !self.stop_signal.load(Ordering::Relaxed)) {
            self.max_depth += 1;
            index += 1;

            if index >= 127 {
                break;
            }

            let results = self.search(board, self.max_depth, 0, WORST_EVAL, BEST_EVAL);

            let finished = results.2;

            if finished || best_move.is_none() {
                (eval, best_move) = (results.0, results.1);
            } else {
                self.max_depth -= 1;
            }
        }

        (eval, best_move)
    }

    /// Searches for a move with the highest evaluation with a fixed depth and a hard time limit.
    pub fn search(&mut self, board: &Board, depth: usize, ply: usize, mut alpha: i32, beta: i32) -> (i32, Option<Move>, bool) {
        if board.half_move_counter >= 100 {
            return (0, None, true); // 50 move repetition.
        }
        
        if depth == 0 {
            return (eval::evaluate_board(board), None, true);
        }

        let mut moves = ArrayVec::new();
        board.generate_moves(&mut moves);
        order_moves(board, self, &mut moves);

        let mut best_move = None;
        let mut best_eval = WORST_EVAL;

        let mut valid_moves = 0;
        let mut last_valid_move: Option<Move> = None;

        for piece_move in moves.iter() {
            let Some(board) = board.make_move(piece_move, false) else { continue; };
            self.nodes += 1;

            valid_moves += 1;
            last_valid_move = Some(*piece_move);

            // let mut transpositions = 0;
            // for position in self.past_boards.iter() { // NOTE: The vector is cleared every time the half-move counter resets.
            //     if board.zobrist_key == *position {
            //         transpositions += 1;

            //         if transpositions >= 2 {
            //             break;
            //         }
            //     }
            // }

            // if transpositions >= 2 {continue;}

            if self.past_boards.iter().filter(|p| **p == board.zobrist_key).count() == 2 {
                continue;
            }

            let (mut eval, _, _) = self.search(&board, depth - 1, ply + 1, -beta, -alpha);
            eval *= -1;

            if eval > best_eval || best_move.is_none() {
                best_eval = eval;
                best_move = Some(*piece_move);
            }

            if eval >= beta {
                return (beta, best_move, true);
            }

            if eval > alpha {
                alpha = eval;
            }

            if self.stop_signal.load(Ordering::Relaxed) || self.timer.elapsed() > self.time_limit {
                if best_move.is_none() {
                    best_move = Some(*piece_move);
                }

                return (best_eval, best_move, false);
            }
        }

        if valid_moves == 0 {
            if board.in_check(board.side_to_move) {
                return (WORST_EVAL + ply as i32, None, true); // Checkmate.
            } else {
                return (0, None, true); // Stalemate.
            }
        }

        if best_move.is_none() && self.max_depth == depth {
            best_move = last_valid_move;

            if best_move.is_none() {
                panic!("null move in search ({} real moves, {} valid moves)", moves.len(), valid_moves);
            }
        }

        (best_eval, best_move, true)
    }
}