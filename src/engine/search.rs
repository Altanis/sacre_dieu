use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::{Duration, Instant}};

use arrayvec::ArrayVec;

use crate::utils::{board::Board, consts::{BEST_EVAL, MAX_DEPTH, SHALLOWEST_PROVEN_LOSS, WORST_EVAL}, piece_move::{order_moves, Move}, transposition_table::{EvaluationType, TTEntry, TranspositionTable}};
use super::eval;

pub struct Searcher {
    /// The past board positions, represented as zobrist hashes.
    pub past_boards: Vec<u64>,
    /// A table of previous searches and their evaluations.
    pub transposition_table: TranspositionTable,
    
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
    /// The best move searched.
    pub best_move: Option<Move>,
    /// Whether or not the search finished.
    pub finished: bool
}

impl Searcher {
    /// Initializes a new searcher.
    pub fn new(time_limit: Duration, depth: usize, stop_signal: Arc<AtomicBool>) -> Self {
        Searcher {
            past_boards: Vec::new(),
            transposition_table: TranspositionTable::from_mb(16),

            time_limit,
            timer: Instant::now(),
            max_depth: depth,
            stop_signal,

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

            self.nodes = 0;
            self.finished = true;
            self.max_depth += 1;

            let latest_eval = self.search(board, self.max_depth, 0, WORST_EVAL, BEST_EVAL);

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
        // check if this is better
        // if ply > 0 {
        //     if board.half_move_counter >= 100 || self.past_boards.iter().filter(|p| **p == board.zobrist_key).count() == 2 {
        //         return 0; // 50 move repetition or threefold repetition.
        //     }
        // }

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

        let (mut best_score, mut best_move) = (WORST_EVAL, None);

        for piece_move in moves.iter() {
            let Some(board) = board.make_move(piece_move, false) else { continue; };
            
            self.nodes += 1;
            has_moves = true;

            if ply != 0 && self.past_boards.iter().filter(|p| **p == board.zobrist_key).count() == 2 {
                return 0;
            }

            let score = -self.search(&board, depth - 1, ply + 1, -beta, -alpha);

            if score > best_score {
                best_score = score;
            }

            if score > alpha {
                alpha = score;
                best_move = Some(*piece_move);

                if ply == 0 {
                    self.best_move = Some(*piece_move);
                }
            }

            if score >= beta {
                break;
            }

            if self.stop_signal.load(Ordering::Relaxed) || self.timer.elapsed() > self.time_limit {
                self.finished = false;
                return best_score;
            }
        }

        if !has_moves {
            if board.in_check(board.side_to_move) {
                return SHALLOWEST_PROVEN_LOSS + ply as i32; // Checkmate.
            } else {
                return 0; // Stalemate.
            }
        }

        best_score
    }

    pub fn quiescence_search(&mut self, board: &Board, mut alpha: i32, beta: i32) -> i32 {
        let eval = eval::evaluate_board(board);
        if eval >= beta {
            return eval;
        }

        alpha = alpha.max(eval);

        let mut moves = ArrayVec::new();
        board.generate_moves(&mut moves, true);
        order_moves(board, self, &mut moves);

        let mut best_score = eval;

        for piece_move in moves.iter() {
            let Some(board) = board.make_move(piece_move, false) else { continue; };
            self.nodes += 1;

            let score = -self.quiescence_search(&board, -beta, -alpha);

            if score > best_score {
                best_score = score;
            }

            if score > alpha {
                alpha = score;
            }

            if score >= beta {
                break;
            }
        }

        best_score
    }
}