use arrayvec::ArrayVec;

use crate::engine::search::Searcher;

use super::{board::{Bitboard, Board}, consts::{BLACK_PAWN_MASK, MAX_LEGAL_MOVES, PIECE_SQUARE_TABLE, WHITE_PAWN_MASK}, piece::{PieceColor, PieceType, Tile}};

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

/// Orders moves based off guesses.
pub fn order_moves(board: &Board, searcher: &Searcher, moves: &mut ArrayVec<Move, MAX_LEGAL_MOVES>) {
    let mut scores: ArrayVec<i32, MAX_LEGAL_MOVES> = ArrayVec::new();

    for piece_move in moves.iter() {
        let mut score: i32 = 0;

        let initial_piece = board.board[piece_move.initial.index()]
            .clone()
            .expect("expected piece on initial square");

        // MVV-LVA
        if let Some(piece) = board.board[piece_move.end.index()].as_ref() {
            score = 100 * piece.piece_type.get_value() as i32 - initial_piece.piece_type.get_value() as i32;
        }

        // A super naive SEE: I will revisit this later.
        // Penalizes movement onto a tile controlled by a pawn.
        let enemy_pawns = board.colored_piece(PieceType::Pawn, !board.side_to_move);
        let pawn_attacks = Bitboard::new(match !board.side_to_move {
            PieceColor::White => BLACK_PAWN_MASK[piece_move.end.index()].1,
            PieceColor::Black => WHITE_PAWN_MASK[piece_move.end.index()].1,
        }) & enemy_pawns;

        if pawn_attacks != Bitboard::ZERO {
            score -= initial_piece.piece_type.get_value() as i32;
        }

        scores.push(score);
    }

    let mut combined: ArrayVec<(_, _), MAX_LEGAL_MOVES> = scores.iter().copied().zip(moves.iter().copied()).collect();
    combined.sort_by_key(|&(score, _)| std::cmp::Reverse(score));

    for (i, (_, piece_move)) in combined.into_iter().enumerate() {
        moves[i] = piece_move;
    }
}