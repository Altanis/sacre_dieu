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
            let capture_bucket = if Self::static_exchange_evaluation(board, piece_move, -108) { Self::GOOD_CAPTURE } else { Self::BAD_CAPTURE };

            return capture_bucket + mvv_lva;
        }

        let is_quiet = !qsearch && piece_move.flags != MoveFlags::EnPassant && board.board[piece_move.end.index()].is_none();
        if is_quiet {
            // History + Killer Heuristics
            let killer_move = self.get_killer(ply);
            let history_score = self.get_history(board, piece_move);

            if killer_move == Some(piece_move) {
                return Self::KILLER_MOVE + history_score;
            } else {
                return Self::QUIET_MOVE + history_score;
            }
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
    const HASH_MOVE: i32 = 100_000_000;
    const GOOD_CAPTURE: i32 = 20_000_000;
    const KILLER_MOVE: i32 = 15_000_000;
    const COUNTER_MOVE: i32 = 10_000_000;
    const QUIET_MOVE: i32 = 5_000_000;
    const BAD_CAPTURE: i32 = 0;

    const SEE_VALUES: [i32; 6] = [PAWN_VALUE, KNIGHT_VALUE, BISHOP_VALUE, ROOK_VALUE, QUEEN_VALUE, KING_VALUE];
}

#[cfg(test)]
mod tests {
    use crate::utils::{board::Board, piece_move::MoveSorter};
    use super::Move;

    const SEE: &str = "6k1/1pp4p/p1pb4/6q1/3P1pRr/2P4P/PP1Br1P1/5RKN w - - | f1f4 | -100 | P - R + B
5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - - | d6f4 | 0 | -N + B
4R3/2r3p1/5bk1/1p1r3p/p2PR1P1/P1BK1P2/1P6/8 b - - | h5g4 | 0
4R3/2r3p1/5bk1/1p1r1p1p/p2PR1P1/P1BK1P2/1P6/8 b - - | h5g4 | 0
4r1k1/5pp1/nbp4p/1p2p2q/1P2P1b1/1BP2N1P/1B2QPPK/3R4 b - - | g4f3 | 0
2r1r1k1/pp1bppbp/3p1np1/q3P3/2P2P2/1P2B3/P1N1B1PP/2RQ1RK1 b - - | d6e5 | 100 | P
7r/5qpk/p1Qp1b1p/3r3n/BB3p2/5p2/P1P2P2/4RK1R w - - | e1e8 | 0
6rr/6pk/p1Qp1b1p/2n5/1B3p2/5p2/P1P2P2/4RK1R w - - | e1e8 | -500 | -R
7r/5qpk/2Qp1b1p/1N1r3n/BB3p2/5p2/P1P2P2/4RK1R w - - | e1e8 | -500 | -R
6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - - | f7f8q | 200 | B - P
6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - - | f7f8n | 200 | N - P
7R/5P2/8/8/6r1/3K4/5p2/4k3 w - - | f7f8q | 800 | Q - P
7R/5P2/8/8/6r1/3K4/5p2/4k3 w - - | f7f8b | 200 | B - P
7R/4bP2/8/8/1q6/3K4/5p2/4k3 w - - | f7f8r | -100 | -P
8/4kp2/2npp3/1Nn5/1p2PQP1/7q/1PP1B3/4KR1r b - - | h1f1 | 0
8/4kp2/2npp3/1Nn5/1p2P1P1/7q/1PP1B3/4KR1r b - - | h1f1 | 0
2r2r1k/6bp/p7/2q2p1Q/3PpP2/1B6/P5PP/2RR3K b - - | c5c1 | 100 | R - Q + R
r2qk1nr/pp2ppbp/2b3p1/2p1p3/8/2N2N2/PPPP1PPP/R1BQR1K1 w kq - | f3e5 | 100 | P
6r1/4kq2/b2p1p2/p1pPb3/p1P2B1Q/2P4P/2B1R1P1/6K1 w - - | f4e5 | 0
3q2nk/pb1r1p2/np6/3P2Pp/2p1P3/2R4B/PQ3P1P/3R2K1 w - h6 | g5h6ep | 0
3q2nk/pb1r1p2/np6/3P2Pp/2p1P3/2R1B2B/PQ3P1P/3R2K1 w - h6 | g5h6ep | 100 | P
2r4r/1P4pk/p2p1b1p/7n/BB3p2/2R2p2/P1P2P2/4RK2 w - - | c3c8 | 500 | R
2r4k/2r4p/p7/2b2p1b/4pP2/1BR5/P1R3PP/2Q4K w - - | c3c5 | 300 | B
8/pp6/2pkp3/4bp2/2R3b1/2P5/PP4B1/1K6 w - - | g2c6 | -200 | P - B
4q3/1p1pr1k1/1B2rp2/6p1/p3PP2/P3R1P1/1P2R1K1/4Q3 b - - | e6e4 | -400 | P - R
4q3/1p1pr1kb/1B2rp2/6p1/p3PP2/P3R1P1/1P2R1K1/4Q3 b - - | h7e4 | 100 | P
3r3k/3r4/2n1n3/8/3p4/2PR4/1B1Q4/3R3K w - - | d3d4 | -100 | P - R + N - P + N - B + R - Q + R
1k1r4/1ppn3p/p4b2/4n3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - | d3e5 | 100 | N - N + B - R + N
1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - | d3e5 | -200 | P - N
rnb2b1r/ppp2kpp/5n2/4P3/q2P3B/5R2/PPP2PPP/RN1QKB2 w Q - | h4f6 | 100 | N - B + P
r2q1rk1/2p1bppp/p2p1n2/1p2P3/4P1b1/1nP1BN2/PP3PPP/RN1QR1K1 b - - | g4f3 | 0 | N - B
r1bqkb1r/2pp1ppp/p1n5/1p2p3/3Pn3/1B3N2/PPP2PPP/RNBQ1RK1 b kq - | c6d4 | 0 | P - N + N - P
r1bq1r2/pp1ppkbp/4N1p1/n3P1B1/8/2N5/PPP2PPP/R2QK2R w KQ - | e6g7 | 0 | B - N
r1bq1r2/pp1ppkbp/4N1pB/n3P3/8/2N5/PPP2PPP/R2QK2R w KQ - | e6g7 | 300 | B
rnq1k2r/1b3ppp/p2bpn2/1p1p4/3N4/1BN1P3/PPP2PPP/R1BQR1K1 b kq - | d6h2 | -200 | P - B
rn2k2r/1bq2ppp/p2bpn2/1p1p4/3N4/1BN1P3/PPP2PPP/R1BQR1K1 b kq - | d6h2 | 100 | P
r2qkbn1/ppp1pp1p/3p1rp1/3Pn3/4P1b1/2N2N2/PPP2PPP/R1BQKB1R b KQq - | g4f3 | 100 | N - B + P
rnbq1rk1/pppp1ppp/4pn2/8/1bPP4/P1N5/1PQ1PPPP/R1B1KBNR b KQ - | b4c3 | 0 | N - B
r4rk1/3nppbp/bq1p1np1/2pP4/8/2N2NPP/PP2PPB1/R1BQR1K1 b - - | b6b2 | -800 | P - Q
r4rk1/1q1nppbp/b2p1np1/2pP4/8/2N2NPP/PP2PPB1/R1BQR1K1 b - - | f6d5 | -200 | P - N
1r3r2/5p2/4p2p/2k1n1P1/2PN1nP1/1P3P2/8/2KR1B1R b - - | b8b3 | -400 | P - R
1r3r2/5p2/4p2p/4n1P1/kPPN1nP1/5P2/8/2KR1B1R b - - | b8b4 | 100 | P
2r2rk1/5pp1/pp5p/q2p4/P3n3/1Q3NP1/1P2PP1P/2RR2K1 b - - | c8c1 | 0 | R - R
5rk1/5pp1/2r4p/5b2/2R5/6Q1/R1P1qPP1/5NK1 b - - | f5c2 | -100 | P - B + R - Q + R
1r3r1k/p4pp1/2p1p2p/qpQP3P/2P5/3R4/PP3PP1/1K1R4 b - - | a5a2 | -800 | P - Q
1r5k/p4pp1/2p1p2p/qpQP3P/2P2P2/1P1R4/P4rP1/1K1R4 b - - | a5a2 | 100 | P
r2q1rk1/1b2bppp/p2p1n2/1ppNp3/3nP3/P2P1N1P/BPP2PP1/R1BQR1K1 w - - | d5e7 | 0 | B - N
rnbqrbn1/pp3ppp/3p4/2p2k2/4p3/3B1K2/PPP2PPP/RNB1Q1NR w - - | d3e4 | 100 | P
rnb1k2r/p3p1pp/1p3p1b/7n/1N2N3/3P1PB1/PPP1P1PP/R2QKB1R w KQkq - | e4d6 | -200 | -N + P
r1b1k2r/p4npp/1pp2p1b/7n/1N2N3/3P1PB1/PPP1P1PP/R2QKB1R w KQkq - | e4d6 | 0 | -N + N
2r1k2r/pb4pp/5p1b/2KB3n/4N3/2NP1PB1/PPP1P1PP/R2Q3R w k - | d5c6 | -300 | -B
2r1k2r/pb4pp/5p1b/2KB3n/1N2N3/3P1PB1/PPP1P1PP/R2Q3R w k - | d5c6 | 0 | -B + B
2r1k3/pbr3pp/5p1b/2KB3n/1N2N3/3P1PB1/PPP1P1PP/R2Q3R w - - | d5c6 | -300 | -B + B - N
5k2/p2P2pp/8/1pb5/1Nn1P1n1/6Q1/PPP4P/R3K1NR w KQ - | d7d8q | 800 | (Q - P)
r4k2/p2P2pp/8/1pb5/1Nn1P1n1/6Q1/PPP4P/R3K1NR w KQ - | d7d8q | -100 | (Q - P) - Q
5k2/p2P2pp/1b6/1p6/1Nn1P1n1/8/PPP4P/R2QK1NR w KQ - | d7d8q | 200 | (Q - P) - Q + B
4kbnr/p1P1pppp/b7/4q3/7n/8/PP1PPPPP/RNBQKBNR w KQk - | c7c8q | -100 | (Q - P) - Q
4kbnr/p1P1pppp/b7/4q3/7n/8/PPQPPPPP/RNB1KBNR w KQk - | c7c8q | 200 | (Q - P) - Q + B
4kbnr/p1P1pppp/b7/4q3/7n/8/PPQPPPPP/RNB1KBNR w KQk - | c7c8q | 200 | (Q - P)
4kbnr/p1P4p/b1q5/5pP1/4n3/5Q2/PP1PPP1P/RNB1KBNR w KQk f6 | g5f6ep | 0 | P - P
4kbnr/p1P4p/b1q5/5pP1/4n2Q/8/PP1PPP1P/RNB1KBNR w KQk f6 | g5f6ep | 0 | P - P
1n2kb1r/p1P4p/2qb4/5pP1/4n2Q/8/PP1PPP1P/RNB1KBNR w KQk - | c7b8q | 200 | N + (Q - P) - Q
rnbqk2r/pp3ppp/2p1pn2/3p4/3P4/N1P1BN2/PPB1PPPb/R2Q1RK1 w kq - | g1h2 | 300 | B
3N4/2K5/2n5/1k6/8/8/8/8 b - - | c6d8 | 0 | N - N
3n3r/2P5/8/1k6/8/8/3Q4/4K3 w - - | c7d8q | 700 | (N + Q - P) - Q + R
r2n3r/2P1P3/4N3/1k6/8/8/8/4K3 w - - | e6d8 | 300 | N
8/8/8/1k6/6b1/4N3/2p3K1/3n4 w - - | e3d1 | 0 | N - N
8/8/1k6/8/8/2N1N3/4p1K1/3n4 w - - | c3d1 | 100 | N - (N + Q - P) + Q
r1bqk1nr/pppp1ppp/2n5/1B2p3/1b2P3/5N2/PPPP1PPP/RNBQK2R w KQkq - | e1g1 | 0";

    #[test]
    fn test_see_1() {
        let suite: Vec<(&str, &str, i32, bool)> = vec![
            ("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - - 0 1", "e1e5", 0, true),
            ("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - 0 1", "d3e5", 0, false),
            ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", "g2h3", 0, true),
            ("k3r3/8/8/4p3/8/2B5/1B6/K7 w - - 0 1", "c3e5", 0, true),
            ("4kbnr/p1P4p/b1q5/5pP1/4n3/5Q2/PP1PPP1P/RNB1KBNR w KQk f6 0 1", "g5f6ep", 0, true),
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

    fn parse_line(line: &str) -> (String, String, i32) {
        // Split the line by spaces
        let parts: Vec<&str> = line.split('|').collect();
    
        // Ensure the line has the expected number of parts
        if parts.len() < 3 {
            panic!("mehwek");
        }
    
        // Extract the parts
        let fen = parts[0].trim().to_string();
        let piece_move = parts[1].trim().to_string();
        
        // Remove leading/trailing spaces and parse the threshold
        let threshold_part = parts[2].trim();
        let threshold = threshold_part.split('|').next().unwrap_or("-100").trim().parse::<i32>().unwrap_or(0);
    
        (fen, piece_move, threshold)
    }

    #[test]
    fn test_see_2() {
        // Ensure SEE values are [100, 300, 300, 500, 900, 0] in this case.
        // otherwise it will fail.

        let mut elines = vec![];
        for line in SEE.split('\n').collect::<Vec<_>>() {
            elines.push(parse_line(line));
        }
    
        let len = elines.len();
        for (i, (fen, piece_move, threshold)) in elines.into_iter().enumerate() {
            let board = Board::new(fen.as_str());
            let piece_move = Move::from_uci(piece_move.as_str());
    
            if MoveSorter::static_exchange_evaluation(&board, piece_move, threshold) != true {
                panic!("({}/{}) Assertion failed for SEE.\nBoard FEN: {}\nMove: {}\nThreshold: {}\nExpected Result: {}", i + 1, len, fen, piece_move.to_uci(), threshold, true);
            } else {
                println!("({}/{}) passed.", i + 1, len);
            }
        }
    }
}