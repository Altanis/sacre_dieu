use arrayvec::ArrayVec;

use crate::engine::search::Searcher;

use super::{board::{Bitboard, Board}, consts::{BEST_EVAL, BLACK_PAWN_MASK, MAX_DEPTH, MAX_LEGAL_MOVES, WHITE_PAWN_MASK, WORST_EVAL}, piece::{PieceColor, PieceType, Tile}};

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
        if uci.len() != 4 && uci.len() != 5 {
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
    pub fn order_moves(&self, board: &Board, searcher: &Searcher, moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>, ply: usize, qsearch: bool) {
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
            let mvv_lva = 100 * piece.piece_type.get_value() as i32 - initial_piece.piece_type.get_value() as i32;
            return Self::CAPTURE_MOVE + mvv_lva;
        }

        // let is_quiet = !qsearch && piece_move.flags != MoveFlags::EnPassant && board.board[piece_move.end.index()].is_none();
        // if is_quiet {
        //     // History and Killer Heuristics
        //     // let killer_move = self.get_killer(ply);
        //     let history_score = self.get_history(board, piece_move);

        //     // if killer_move == Some(piece_move) {
        //         // return Self::KILLER_MOVE + history_score;
        //     // } else {
        //         return Self::QUIET_MOVE + history_score;
        //     // }
        // }

        0
    }
}

impl MoveSorter {
    const HASH_MOVE: i32 = 100_000_000;
    const CAPTURE_MOVE: i32 = 0;
    const KILLER_MOVE: i32 = -100_000;
    const QUIET_MOVE: i32 = -100_000_000;
}