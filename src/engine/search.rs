use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::{Duration, Instant}};

use arrayvec::ArrayVec;

use crate::utils::{board::Board, consts::{BEST_EVAL, LMR_MOVE_THRESHOLD, LMR_REDUCTION_BASE, LMR_REDUCTION_DIVISOR, LMR_REDUCTION_TABLE, MAX_DEPTH, RFP_DEPTH, RFP_THRESHOLD, SHALLOWEST_PROVEN_LOSS, WORST_EVAL}, piece_move::{Move, MoveArray, MoveFlags, MoveSorter}, transposition_table::{EvaluationType, TTEntry, TranspositionTable}};
use super::eval;

/// An entry in the search stack.
#[derive(Debug, Default, Clone)]
pub struct SearchEntry {
    /// The killer move at the ply.
    pub killer_move: Option<Move>,
    /// The static evaluation at the ply.
    pub static_eval: i32
}

pub struct Searcher {
    /// The past board positions, represented as zobrist hashes.
    pub past_boards: Vec<u64>,
    /// A table of previous searches and their evaluations.
    pub transposition_table: TranspositionTable,
    /// A table of search information, indexed by ply.
    pub search_stack: [SearchEntry; MAX_DEPTH + 4],
    /// A struct which sorts moves.
    pub move_sorter: MoveSorter,
    
    /// The soft time constraint of the search.
    pub soft_tm: Duration,
    /// The hard time constraint of the search.
    pub hard_tm: Duration,
    /// The timer associated with the search.
    pub timer: Instant,
    /// The current depth of the search.
    pub depth: usize,
    /// The maximum depth of the search.
    pub max_depth: usize,
    /// A boolean signalling when to stop a search.
    pub stop_signal: Arc<AtomicBool>,
    
    /// The number of nodes searched.
    pub nodes: usize,
    /// The best move searched.
    pub best_move: Option<Move>
}

impl Searcher {
    /// Initializes a new searcher.
    pub fn new(soft_tm: Duration, hard_tm: Duration, max_depth: usize, stop_signal: Arc<AtomicBool>) -> Self {
        Searcher {
            past_boards: Vec::new(),
            transposition_table: TranspositionTable::from_mb(16),
            search_stack: std::array::from_fn(|_| SearchEntry::default()),
            move_sorter: MoveSorter::new(),

            soft_tm,
            hard_tm,
            timer: Instant::now(),
            depth: 0,
            max_depth,
            stop_signal,

            nodes: 0,
            best_move: None
        }
    }

    /// Whether or not the search has been cancelled.
    pub fn search_cancelled(&self) -> bool {
        self.stop_signal.load(Ordering::Relaxed) || self.timer.elapsed() > self.hard_tm
    }

    /// Gets an entry at a ply in the search stack.
    pub fn get_search_entry(&self, ply: usize) -> Option<SearchEntry> {
        self.search_stack.get(ply).cloned()
    }

    /// Updates a killer move at a ply in the search stack.
    pub fn update_killer(&mut self, killer_move: Option<Move>, ply: usize) {
        self.search_stack[ply].killer_move = killer_move;
    }

    /// Updates a static eval at a ply in the search stack.
    pub fn update_static_eval(&mut self, eval: i32, ply: usize) {
        self.search_stack[ply].static_eval = eval;
    }

    /// Searches for a move with a time constraint.
    pub fn search_timed(&mut self, board: &Board) -> i32 {
        self.timer = std::time::Instant::now();
        let (mut eval, mut best_move) = (0, None);

        self.depth = 0;
        for _ in 0..=self.max_depth {
            // Soft Time Control
            if self.timer.elapsed() >= self.soft_tm {
                break;
            }

            self.depth += 1;
            let score = self.aspiration_windows(board, self.depth, eval);
            // let score = self.search::<true>(board, self.depth, 0, WORST_EVAL, BEST_EVAL);

            if self.search_cancelled() {
                break;
            } else {
                eval = score;
                best_move = self.best_move;
            } 
        }

        self.best_move = best_move;

        eval
    }

    /// Iteratively reduces the window for the search to yield more cutoffs.
    pub fn aspiration_windows(&mut self, board: &Board, depth: usize, prev_score: i32) -> i32 {
        let mut delta = 25;
        let (mut alpha, mut beta) = (WORST_EVAL, BEST_EVAL);

        if depth >= 4 {
            alpha = prev_score - delta;
            beta = prev_score + delta;
        }

        loop {
            let search_score = self.search::<true>(board, depth, 0, alpha, beta);
            if self.search_cancelled() {
                return search_score;
            }

            if search_score <= alpha {
                alpha -= delta;
            } else if search_score >= beta {
                beta += delta;
            } else {
                return search_score;
            }

            delta *= 2;
        }
    }

    /// Searches for a move with the highest evaluation with a fixed depth and a hard time limit.
    pub fn search<const PV: bool>(&mut self, old_board: &Board, depth: usize, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        self.update_killer(None, ply + 2);

        if ply > 0 && (old_board.half_move_counter >= 100 || self.past_boards.iter().filter(|p| **p == old_board.zobrist_key).count() == 2) {
            return 0; // 50 move repetition or threefold repetition.
        }

        if depth == 0 {
            return self.quiescence_search(old_board, ply, alpha, beta);
        }

        if !PV && ply > 0 {
            if let Some(entry) = self.transposition_table.get(old_board.zobrist_key) {
                if entry.zobrist_key == old_board.zobrist_key && entry.depth >= depth {
                    match entry.evaluation_type {
                        EvaluationType::Exact => return entry.evaluation,
                        EvaluationType::UpperBound if entry.evaluation <= alpha => return entry.evaluation,
                        EvaluationType::LowerBound if entry.evaluation >= beta => return entry.evaluation,
                        _ => {}
                    }
                }
            }
        }

        let in_check = old_board.in_check(old_board.side_to_move);
        let static_eval = eval::evaluate_board(old_board);

        self.update_static_eval(static_eval, ply);

        let improving = if in_check {
            false
        } else {
            let current_ply = self.get_search_entry(ply).expect("couldnt find search entry tf").static_eval;
            let two_ply_back = self.get_search_entry(ply - 2).unwrap_or(SearchEntry { static_eval: WORST_EVAL, killer_move: None }).static_eval;
            let four_ply_back = self.get_search_entry(ply - 2).unwrap_or(SearchEntry { static_eval: WORST_EVAL, killer_move: None }).static_eval;

            current_ply > two_ply_back || current_ply > four_ply_back
        };

        // Reverse Futility Pruning
        if !PV && !in_check && depth < RFP_DEPTH && static_eval - (RFP_THRESHOLD * (depth - improving as usize)) as i32 >= beta {
            return static_eval;
        }

        // Null Move Pruning
        if !PV && !in_check && static_eval >= beta {
            let depth = (depth as isize - 3) - (depth as isize / 3);

            let nmp_board = old_board.make_null_move();
            let nmp_score = -self.search::<false>(&nmp_board, depth.max(0) as usize, ply + 1, -beta, -alpha);
            if nmp_score >= beta {
                return nmp_score;
            }
        }

        let mut moves = ArrayVec::new();
        old_board.generate_moves(&mut moves, false);
        self.move_sorter.order_moves(old_board, self, &mut moves, ply, false);

        let mut quiet_moves: MoveArray = ArrayVec::new();
        let mut num_moves = 0;

        let (mut best_score, mut best_move) = (WORST_EVAL, None);
        let mut evaluation_type = EvaluationType::UpperBound;

        for piece_move in moves.iter() {
            let is_quiet = piece_move.flags != MoveFlags::EnPassant && old_board.board[piece_move.end.index()].is_none();

            // Late Move Pruning
            if !PV && is_quiet && depth <= 5 && num_moves >= (8 * depth) / (2 - improving as usize) {
                continue;
            }

            let Some(board) = old_board.make_move(piece_move, false) else { continue; };

            self.nodes += 1;
            num_moves += 1;

            let extension = if board.in_check(board.side_to_move) { 1 } else { 0 };

            let mut score = 0;

            if num_moves == 1 {
                // Full Window Search
                score = -self.search::<PV>(&board, depth - 1 + extension, ply + 1, -beta, -alpha);
            } else {
                let reduction = if !PV && !in_check && num_moves > LMR_MOVE_THRESHOLD {
                    LMR_REDUCTION_TABLE[depth][num_moves]
                } else {
                    0_f32
                };

                // Null Window Search
                score = -self.search::<false>(&board, (depth as f32 - 1.0 - reduction + extension as f32).max(0.0) as usize, ply + 1, -alpha - 1, -alpha);

                if score > alpha && (score < beta || reduction > 0.0) {
                    // Null Window Search failed, resort to Full Window Search
                    score = -self.search::<PV>(&board, depth - 1 + extension, ply + 1, -beta, -alpha);
                }
            }

            if self.search_cancelled() {
                return best_score;
            }

            if score > best_score {
                best_score = score;
            }

            if score > alpha {
                evaluation_type = EvaluationType::Exact;

                alpha = score;
                best_move = Some(*piece_move);

                if ply == 0 {
                    self.best_move = Some(*piece_move);
                }
            }

            if score >= beta {
                if is_quiet {
                    // History Heuristic
                    let bonus = (depth * depth) as i32;
                    self.move_sorter.update_history(old_board, *piece_move, bonus);

                    for old_move in quiet_moves.iter() {
                        self.move_sorter.update_history(old_board, *old_move, -bonus);
                    }

                    // Killer Heuristic
                    self.update_killer(Some(*piece_move), ply);
                }

                evaluation_type = EvaluationType::LowerBound;
                break;
            }

            if is_quiet {
                quiet_moves.push(*piece_move);
            }
        }

        if num_moves == 0 {
            if old_board.in_check(old_board.side_to_move) {
                return SHALLOWEST_PROVEN_LOSS + ply as i32; // Checkmate.
            } else {
                return 0; // Stalemate.
            }
        }

        if !self.search_cancelled() {
            self.transposition_table.store(old_board.zobrist_key, TTEntry { zobrist_key: old_board.zobrist_key, depth, evaluation: best_score, evaluation_type, best_move });
        }

        best_score
    }

    pub fn quiescence_search(&mut self, board: &Board, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        let eval = eval::evaluate_board(board);
        if eval >= beta {
            return eval;
        }

        alpha = alpha.max(eval);

        let mut moves = ArrayVec::new();
        board.generate_moves(&mut moves, true);
        self.move_sorter.order_moves(board, self, &mut moves, ply, true);

        let mut best_score = eval;

        for piece_move in moves.iter() {
            let Some(board) = board.make_move(piece_move, false) else { continue; };
            self.nodes += 1;

            let score = -self.quiescence_search(&board, ply + 1, -beta, -alpha);

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