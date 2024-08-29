use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::{Duration, Instant}};

use arrayvec::ArrayVec;

use crate::utils::{board::Board, consts::{BEST_EVAL, MAX_DEPTH, SHALLOWEST_PROVEN_LOSS, WORST_EVAL}, piece_move::{order_moves, Move}};
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
    /// The past board positions, represented as zobrist hashes.
    pub past_boards: Vec<u64>,
    /// The number of nodes searched.
    pub nodes: usize,
    /// The best move searched.
    pub best_move: Option<Move>,
    /// Whether or not the search finished.
    pub finished: bool
}

impl Searcher {
    /// Initializes a new searcher.
    pub fn new(time_limit: Duration, depth: usize, stop_signal: Arc<AtomicBool>) -> Self {
        Searcher {
            time_limit,
            timer: Instant::now(),
            max_depth: depth,
            stop_signal,
            past_boards: Vec::new(),
            nodes: 0,
            best_move: None,
            finished: true
        }
    }

    /// Searches for a move with a time constraint.
    pub fn search_timed(&mut self, board: &Board) -> i32 {
        self.timer = std::time::Instant::now();
        let (mut eval, mut best_move) = (0, None);

        self.max_depth = 0;
        for _ in 0..=MAX_DEPTH {
            if self.timer.elapsed() > self.time_limit || self.stop_signal.load(Ordering::Relaxed) {
                break;
            }

            self.max_depth += 1;
            self.finished = true;

            let latest_eval = -self.search(board, self.max_depth, 0, WORST_EVAL, BEST_EVAL);

            if self.finished {
                eval = latest_eval;
                best_move = self.best_move;
            } else {
                break;
            }
        }

        self.best_move = best_move;

        eval
    }

    /// Searches for a move with the highest evaluation with a fixed depth and a hard time limit.
    pub fn search(&mut self, board: &Board, depth: usize, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        if board.half_move_counter >= 100 {
            return 0; // 50 move repetition.
        }
        
        if depth == 0 {
            return self.quiescence_search(board, alpha, beta);
        }

        let mut moves = ArrayVec::new();
        board.generate_moves(&mut moves, false);
        order_moves(board, self, &mut moves);

        let mut has_moves = false;
        for piece_move in moves.iter() {
            let Some(board) = board.make_move(piece_move, false) else { continue; };
            
            self.nodes += 1;
            has_moves = true;

            if self.past_boards.iter().filter(|p| **p == board.zobrist_key).count() == 2 {
                return 0;
            }

            let eval = -self.search(&board, depth - 1, ply + 1, -beta, -alpha);

            if eval >= beta {
                return beta;
            }

            if eval > alpha {
                if ply == 0 {
                    self.best_move = Some(*piece_move);
                }

                alpha = eval;
            }

            if self.stop_signal.load(Ordering::Relaxed) || self.timer.elapsed() > self.time_limit {
                self.finished = false;
                break;
            }
        }

        if !has_moves {
            if board.in_check(board.side_to_move) {
                return SHALLOWEST_PROVEN_LOSS + ply as i32; // Checkmate.
            } else {
                return 0; // Stalemate.
            }
        }

        alpha
    }

    pub fn quiescence_search(&mut self, board: &Board, mut alpha: i32, beta: i32) -> i32 {
        let eval = eval::evaluate_board(board);
        if eval >= beta {
            return beta;
        }
        alpha = alpha.max(eval);

        let mut moves = ArrayVec::new();
        board.generate_moves(&mut moves, true);
        order_moves(board, self, &mut moves);

        for piece_move in moves.iter() {
            let Some(board) = board.make_move(piece_move, false) else { continue; };
            let evaluation = -self.quiescence_search(&board, -beta, -alpha);

            if evaluation >= beta {
                return beta;
            }

            alpha = alpha.max(eval);
        }

        alpha
    }
}