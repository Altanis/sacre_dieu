use arrayvec::ArrayVec;
use strum::IntoEnumIterator;

use crate::engine::search::Searcher;

use super::{board::{Bitboard, Board}, consts::{get_bishop_mask, get_rook_mask, BEST_EVAL, BISHOP_MAGICS, BISHOP_VALUE, BLACK_PAWN_MASK, KING_VALUE, KNIGHT_MASKS, KNIGHT_VALUE, MAX_DEPTH, MAX_LEGAL_MOVES, PAWN_VALUE, QUEEN_VALUE, ROOK_MAGICS, ROOK_VALUE, WHITE_PAWN_MASK, WORST_EVAL}, piece::{PieceColor, PieceType, Tile}};

pub type MoveArray = ArrayVec<Move, MAX_LEGAL_MOVES>;

/// A structure representing a move.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Move {
    /// The tile the piece is moving from.
    pub initial: Tile,
    /// The tile the piece is moving to.
    pub end: Tile,
    /// Any additional metadata associated with the move.
    pub flags: MoveFlags
}

/// An enumeration of different move actions.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum MoveFlags {
    None,
    Castling,
    DoublePush,
    EnPassant,
    KnightPromotion,
    BishopPromotion,
    RookPromotion,
    QueenPromotion
}

impl MoveFlags {
    /// Whether or not the flag is a promotion.
    pub fn is_promotion(&self) -> bool {
        matches!(self, MoveFlags::KnightPromotion | MoveFlags::BishopPromotion | MoveFlags::RookPromotion | MoveFlags::QueenPromotion)
    }
}

impl Move {
    /// Creates a new move.
    pub fn new(initial: Tile, end: Tile, flags: MoveFlags) -> Self {
        Move {
            initial,
            end,
            flags
        }
    }

    /// Creates a new move from a UCI string.
    pub fn from_uci(uci: &str) -> Move {
        if uci.len() != 4 && uci.len() != 5 && uci.len() != 6 {
            panic!("invalid uci move");
        }

        let initial = Tile::from_code(&uci[0..2]);
        let end = Tile::from_code(&uci[2..4]);

        let metadata = match uci.len() {
            5 => match uci.chars().nth(4).expect("Invalid UCI move metadata") {
                'N' | 'n' => MoveFlags::KnightPromotion,
                'B' | 'b' => MoveFlags::BishopPromotion,
                'R' | 'r' => MoveFlags::RookPromotion,
                'Q' | 'q' => MoveFlags::QueenPromotion,
                _ => MoveFlags::None
            },
            6 if uci.ends_with("ep") => MoveFlags::EnPassant,
            _ => MoveFlags::None
        };

        Move::new(initial, end, metadata)
    }

    /// Converts a move to its UCI equivalent.
    pub fn to_uci(&self) -> String {
        let mut cur_code = format!("{}{}", self.initial.get_code(), self.end.get_code());
        cur_code += match self.flags {
            MoveFlags::KnightPromotion => "n",
            MoveFlags::BishopPromotion => "b",
            MoveFlags::RookPromotion => "r",
            MoveFlags::QueenPromotion => "q",
            _ => ""
        };

        cur_code
    }

    /// Gets the value of the piece the move is promoting to.
    pub fn get_promotion_type(&self) -> PieceType {
        match self.flags {
            MoveFlags::KnightPromotion => PieceType::Knight,
            MoveFlags::BishopPromotion => PieceType::Bishop,
            MoveFlags::RookPromotion => PieceType::Rook,
            MoveFlags::QueenPromotion => PieceType::Queen,
            _ => panic!("invalid flag")
        }
    }
}

/// A struct which sorts necessary move ordering
/// score constants and tables of vital move ordering
/// information.
pub struct MoveSorter {
    /// A history table which tracks move scores for quiet beta cutoffs.
    pub history_table: [[[i32; 64]; 64]; 2],
    /// A killer table which tracks quiet moves and their plies if they fail high.
    pub killer_table: [Option<Move>; MAX_DEPTH as usize + 4]
}

impl MoveSorter {
    /// Creates a new move sorter.
    pub fn new() -> Self {
        Self {
            history_table: [[[0; 64]; 64]; 2],
            killer_table: [None; MAX_DEPTH as usize + 4]
        }
    }

    /// Gets a move score from history.
    pub fn get_history(&self, board: &Board, piece_move: Move) -> i32 {
        self.history_table[board.side_to_move as usize][piece_move.initial.index()][piece_move.end.index()]
    }

    /// Updates a move score in the history table.
    pub fn update_history(&mut self, board: &Board, piece_move: Move, bonus: i32) {
        let clamped_bonus = bonus.clamp(-16384, 16384);
        let old_value = self.get_history(board, piece_move);

        self.history_table[board.side_to_move as usize][piece_move.initial.index()][piece_move.end.index()]
            += clamped_bonus - old_value * clamped_bonus.abs() / 16384;
    }

    /// Gets a move from the killer table.
    pub fn get_killer(&self, ply: usize) -> Option<Move> {
        self.killer_table[ply]
    }

    /// Updates a move in the killer table.
    pub fn update_killer(&mut self, killer_move: Option<Move>, ply: usize) {
        self.killer_table[ply] = killer_move;
    }

    /// Orders moves based off guesses.
    pub fn order_moves(&self, board: &Board, searcher: &Searcher, moves: &mut MoveArray, ply: usize, qsearch: bool) {
        let mut scores: ArrayVec<i32, MAX_LEGAL_MOVES> = ArrayVec::new();
        let hash_move = searcher.transposition_table.get(board.zobrist_key).and_then(|entry| {
            if entry.zobrist_key == board.zobrist_key { 
                entry.best_move
            } else {
                None
            }
        });

        for piece_move in moves.iter() {
            scores.push(self.score_move(board, *piece_move, ply, hash_move, qsearch));
        }

        let mut combined: ArrayVec<(_, _), MAX_LEGAL_MOVES> = scores.iter().copied().zip(moves.iter().copied()).collect();
        combined.sort_by_key(|&(score, _)| std::cmp::Reverse(score));

        for (i, (_, piece_move)) in combined.into_iter().enumerate() {
            moves[i] = piece_move;
        }
    }

    /// Scores a move.
    pub fn score_move(&self, board: &Board, piece_move: Move, ply: usize, hash_move: Option<Move>, qsearch: bool) -> i32 {
        if hash_move == Some(piece_move) {
            // Hash Move
            return Self::HASH_MOVE;
        }

        let initial_piece = board.board[piece_move.initial.index()]
            .clone()
            .expect("expected piece on initial square");

        // Capture Move
        if let Some(piece) = board.board[piece_move.end.index()].as_ref() { 
            // MVV-LVA
            let mvv_lva = 100 * piece.piece_type.get_value() - initial_piece.piece_type.get_value();

            // SEE
            let capture_bucket = if Self::static_exchange_evaluation(board, piece_move, 0) { Self::GOOD_CAPTURE } else { Self::BAD_CAPTURE };

            return capture_bucket + mvv_lva;
        }

        let is_quiet = !qsearch && piece_move.flags != MoveFlags::EnPassant && board.board[piece_move.end.index()].is_none();
        if is_quiet {
            // History + Killer Heuristics
            // let killer_move = self.get_killer(ply);
            let history_score = self.get_history(board, piece_move);

            // if killer_move == Some(piece_move) {
                // return Self::KILLER_MOVE;
            // } else {
                return Self::QUIET_MOVE + history_score;
            // }
        }

        0
    }

    /// Evaluates the value of what a move captures.
    pub fn capture_move_value(board: &Board, piece_move: Move) -> i32 {
        let mut value = 0;

        if let Some(piece) = board.board[piece_move.end.index()].as_ref() {
            value = Self::SEE_VALUES[piece.piece_type as usize];
        }
        
        if piece_move.flags == MoveFlags::EnPassant {
            value = Self::SEE_VALUES[PieceType::Pawn as usize];
        } else if piece_move.flags.is_promotion() {
            value += Self::SEE_VALUES[piece_move.get_promotion_type() as usize] - Self::SEE_VALUES[PieceType::Pawn as usize];
        }

        value
    }

    /// Checks if a series of moves results in a specific centipawn gain.
    /// 
    /// Thanks to Andrew Grant for his easy-to-read implementation of SEE.
    pub fn static_exchange_evaluation(board: &Board, piece_move: Move, threshold: i32) -> bool {
        // The next piece about to be captured.
        let mut next_victim = if piece_move.flags.is_promotion() { 
            piece_move.get_promotion_type()
        } else { 
            board.board[piece_move.initial.index()].as_ref().unwrap().piece_type
        };

        // The current balance, where positive values mean winning.
        let mut balance = Self::capture_move_value(board, piece_move) - threshold;
        if balance < 0 { // The position is losing even before any moves are done.
            return false;
        }

        balance -= Self::SEE_VALUES[next_victim as usize];
        if balance >= 0 { // The position is winning even if the piece is captured.
            return true;
        }

        // Get a bitboard of sliding pieces.
        let diagonals = board.piece(PieceType::Bishop) | board.piece(PieceType::Queen);
        let orthogonals = board.piece(PieceType::Rook) | board.piece(PieceType::Queen);

        // Get a bitboard of occupied pieces to incrementally update, then perform moves.
        let mut occupied = board.occupied();
        occupied.clear_bit(piece_move.initial);
        occupied.set_bit(piece_move.end);
        if piece_move.flags == MoveFlags::EnPassant {
            occupied.set_bit(board.en_passant.unwrap());
        }

        // Generate all attackers.
        let mut attackers = piece_move.end.attackers(board, occupied);
        let mut color = !board.side_to_move;

        loop {
            let colored_attackers = attackers & board.color(color);
            if colored_attackers == Bitboard::ZERO { // No more attacks left to evaluate.
                break;
            }

            for piece_type in PieceType::iter() {
                next_victim = piece_type;
                if (colored_attackers & board.piece(next_victim)) != Bitboard::ZERO {
                    break;
                }
            }

            occupied.clear_bit((colored_attackers & board.piece(next_victim)).pop_lsb());

            // Diagonal moves may reveal a diagonal attack.
            if matches!(next_victim, PieceType::Pawn | PieceType::Bishop | PieceType::Queen) {
                let diagonal_attacks = get_bishop_mask(Board::generate_magic_index(&BISHOP_MAGICS[piece_move.end.index()], &occupied)) & diagonals;
                attackers |= diagonal_attacks;
            }

            // Same with orthogonal moves.
            if matches!(next_victim, PieceType::Rook | PieceType::Queen) {
                let orthogonal_attacks = get_rook_mask(Board::generate_magic_index(&ROOK_MAGICS[piece_move.end.index()], &occupied)) & orthogonals;
                attackers |= orthogonal_attacks;
            }

            // Remove old attacks.
            attackers &= occupied;

            // Next turn, swap side to move.
            color = !color;

            // Then, adjust the balance.
            balance = -balance - 1 - Self::SEE_VALUES[next_victim as usize];

            if balance >= 0 { // We win/draw if balance is nonnegative.
                // Make sure the king isn't the next victim, as that's not legal.
                if next_victim == PieceType::King && (attackers & board.color(color) != Bitboard::ZERO) {
                    color = !color; // Force a loss, since the attack was illegal.
                }

                break;
            }
        }

        board.side_to_move != color
    }
}

impl MoveSorter {
    // const HASH_MOVE: i32 = 25000;
    // const GOOD_CAPTURE: i32 = 10;
    // const KILLER_MOVE: i32 = 2;
    // const COUNTER_MOVE: i32 = 1;
    // const QUIET_MOVE: i32 = -30000;
    // const BAD_CAPTURE: i32 = -30001;

    const HASH_MOVE: i32 = 1_000_000;
    const GOOD_CAPTURE: i32 = 900_000;
    const KILLER_MOVE: i32 = 800_000;
    const COUNTER_MOVE: i32 = 700_000;
    const QUIET_MOVE: i32 = 600_000;
    const BAD_CAPTURE: i32 = 0;

    const SEE_VALUES: [i32; 6] = [PAWN_VALUE, KNIGHT_VALUE, BISHOP_VALUE, ROOK_VALUE, QUEEN_VALUE, KING_VALUE];
}

#[cfg(test)]
mod tests {
    use crate::utils::{board::Board, piece_move::MoveSorter};

    use super::Move;

    #[test]
    fn test_see() {
        let suite: Vec<(&str, &str, i32, bool)> = vec![
            ("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - - 0 1", "e1e5", 0, true),
            ("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - 0 1", "d3e5", 0, false),
            ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", "g2h3", 0, true),
            ("k3r3/8/8/4p3/8/2B5/1B6/K7 w - - 0 1", "c3e5", 0, true),
            ("4kbnr/p1P4p/b1q5/5pP1/4n3/5Q2/PP1PPP1P/RNB1KBNR w KQk f6 0 1", "g5f6", 0, true),
            ("6k1/1pp4p/p1pb4/6q1/3P1pRr/2P4P/PP1Br1P1/5RKN w - - 0 1", "f1f4", 0, false),
            ("6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - - 0 1", "f7f8q", 0, true),
            ("r1bqk1nr/pppp1ppp/2n5/1B2p3/1b2P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1", "e1g1", 0, true),
            ("4kbnr/p1P1pppp/b7/4q3/7n/8/PPQPPPPP/RNB1KBNR w KQk - 0 1", "c7c8q", 0, true),
            ("4kbnr/p1P1pppp/b7/4q3/7n/8/PP1PPPPP/RNBQKBNR w KQk - 0 1", "c7c8q", 0, false),
            ("3r3k/3r4/2n1n3/8/3p4/2PR4/1B1Q4/3R3K w - - 0 1", "d3d4", 0, false),
            ("5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - - 0 1", "d6f4", 0, false),
            ("5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - - 0 1", "d6f4", -100, true),
        ];

        for (fen, piece_move, threshold, result) in suite.into_iter() {
            let board = Board::new(fen);
            let piece_move = Move::from_uci(piece_move);

            if MoveSorter::static_exchange_evaluation(&board, piece_move, threshold) != result {
                panic!("Assertion failed for SEE.\nBoard FEN: {}\nMove: {}\nThreshold: {}\nExpected Result: {}", fen, piece_move.to_uci(), threshold, result);
            }
        }
    }
}