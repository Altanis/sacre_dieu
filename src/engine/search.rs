use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::{Duration, Instant}};

use arrayvec::ArrayVec;

use crate::utils::{board::Board, consts::{I32_NEGATIVE_INFINITY, I32_POSITIVE_INFINITY}, piece_move::{order_moves, Move}};
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
    pub nodes: usize
}

impl Searcher {
    /// Initializes a new searcher.
    pub fn new(time_limit: Duration, depth: usize, stop_signal: Arc<AtomicBool>) -> Self {
        Searcher {
            time_limit,
            timer: Instant::now(),
            max_depth: depth,
            stop_signal,
            nodes: 0
        }
    }

    /// Searches for a move with a time constraint.
    pub fn search_timed(&mut self, board: &Board) -> (i32, Option<Move>) {
        self.timer = std::time::Instant::now();
        let (mut eval, mut best_move): (i32, Option<Move>) = (0, None);

        self.max_depth = 0;
        while self.timer.elapsed() <= self.time_limit && !self.stop_signal.load(Ordering::Relaxed) {
            self.max_depth += 1;

            let results = self.search(board, self.max_depth, 0, I32_NEGATIVE_INFINITY, I32_POSITIVE_INFINITY);
            let finished = results.2;

            println!("{}", finished);

            if finished {
                (eval, best_move) = (results.0, results.1);
            } else {
                self.max_depth -= 1;
            }
        }

        (eval, best_move)
    }

    /// Searches for a move with the highest evaluation with a fixed depth and a hard time limit.
    pub fn search(&mut self, board: &Board, depth: usize, ply: usize, mut alpha: i32, beta: i32) -> (i32, Option<Move>, bool) {
        if depth == 0 {
            return (eval::evaluate_board(board), None, true);
        }

        let mut moves = ArrayVec::new();
        board.generate_moves(&mut moves);
        order_moves(board, self, &mut moves);

        if moves.is_empty() {
            if board.in_check(board.side_to_move) {
                return (I32_NEGATIVE_INFINITY + (ply as i32), None, true); // Checkmate.
            } else {
                return (0, None, true); // Stalemate.
            }
        }

        let mut best_move = None;
        let mut best_eval = I32_NEGATIVE_INFINITY;

        for piece_move in moves.iter() {
            let Some(board) = board.make_move(piece_move) else { continue; };
            self.nodes += 1;

            let (mut eval, _, _) = self.search(&board, depth - 1, ply + 1, -beta, -alpha);
            eval *= -1;

            if eval > best_eval || best_move.is_none() {
                best_eval = eval;
                best_move = Some(piece_move.clone());
            }

            if eval >= beta {
                return (beta, best_move, true);
            }

            if eval > alpha {
                alpha = eval;
            }

            if self.stop_signal.load(Ordering::Relaxed) || self.timer.elapsed() > self.time_limit {
                return (best_eval, best_move, false);
            }
        }

        (best_eval, best_move, true)
    }
}