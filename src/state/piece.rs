use std::ops::Not;

use super::{board::{Bitboard, Board, PositionalBitboard}, consts::{BISHOP_MAGICS, BLACK_PAWN_MASK, KING_MASKS, KNIGHT_MASKS, MAX_LEGAL_MOVES, ROOK_MAGICS, WHITE_PAWN_MASK}};

/// An enum representing the type of chess piece.
#[derive(Debug, Clone, PartialEq, strum_macros::EnumCount, strum_macros::EnumIter)]
pub enum PieceType {
    Pawn     = 0b1,
    Knight   = 0b10,
    Bishop   = 0b100,
    Rook     = 0b1000,
    Queen    = 0b10000,
    King     = 0b100000
}

impl TryFrom<u32> for PieceType {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0b1 => Ok(PieceType::Pawn),
            0b10 => Ok(PieceType::Knight),
            0b100 => Ok(PieceType::Bishop),
            0b1000 => Ok(PieceType::Rook),
            0b10000 => Ok(PieceType::Queen),
            0b100000 => Ok(PieceType::King),
            _ => Err("Invalid piece type")
        }
    }
}

impl PieceType {
    /// Generates an index from the piece type.
    pub fn to_index(&self) -> usize {
        match self {
            PieceType::Pawn => 2,
            PieceType::Knight => 3,
            PieceType::Bishop => 4,
            PieceType::Rook => 5,
            PieceType::Queen => 6,
            PieceType::King => 7,
        }
    }
}

/// An enum representing the color of a chess piece.
#[derive(Debug, Default, Clone, Copy, PartialEq, strum_macros::EnumCount)]
pub enum PieceColor {
    #[default]
    White  = 0b1,
    Black  = 0b10
}

impl TryFrom<u32> for PieceColor {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0b1 => Ok(PieceColor::White),
            0b10 => Ok(PieceColor::Black),
            _ => Err("Invalid piece color")
        }
    }
}

impl Not for PieceColor {
    type Output = PieceColor;

    fn not(self) -> Self::Output {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White
        }
    }
}

impl PieceColor {
    /// Generates an index from the color.
    pub fn to_index(&self) -> usize {
        match self {
            PieceColor::White => 0,
            PieceColor::Black => 1    
        }
    }
}

/// A position of a cell in chess.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub rank: u8,
    pub file: u8
}

impl Position {
    /// Instantiates a new position.
    pub fn new(rank: u8, file: u8) -> Position {
        if rank > 7 || file > 7 {
            panic!("invalid position {} {}", rank, file);
        }

        Position { rank, file }
    }

    /// Whether or not a position is valid.
    pub fn is_valid(rank: u8, file: u8) -> bool {
        !(rank > 7 || file > 7)
    }

    /// Returns the square of the position.
    pub fn square(&self) -> usize {
        (self.rank * 8 + self.file) as usize
    }

    /// Returns a transformed position.
    pub fn transform(&self, offset_rank: i8, offset_file: i8) -> Self {
        Position::new((self.rank as i8 + offset_rank) as u8, (self.file as i8 + offset_file) as u8)
    }

    /// Converts a position to a square code.
    pub fn get_code(&self) -> String {
        format!("{}{}", (self.file + b'a') as char, self.rank + 1)
    }

    /// Instantiates a new position from a square code.
    pub fn from_code(code: &str) -> Position {
        if code.len() != 2 {
            panic!("invalid code");
        }

        let rank = code.chars().nth(1).expect("Code doesn't have rank").to_digit(10).expect("Rank can't be converted to number") as u8 - 1;
        let file = code.chars().nth(0).expect("Code doesn't have file") as u8 - b'a';

        Position::new(rank, file)
    }

    /// Whether or not a code is valid.
    pub fn is_code_valid(code: &str) -> bool {
        code.len() == 2 && code.chars().nth(1).and_then(|c| c.to_digit(10)).is_some()
    }    

    /// Whether or not the position is under attack from a specific side.
    pub fn is_under_attack(&self, board: &Board, enemy_side: PieceColor) -> bool {
        let enemy_pawns = board.piece_bitboard[PieceType::Pawn.to_index()] & board.piece_bitboard[enemy_side.to_index()];
        let enemy_knights = board.piece_bitboard[PieceType::Knight.to_index()] & board.piece_bitboard[enemy_side.to_index()];
        let enemy_bishops = (board.piece_bitboard[PieceType::Bishop.to_index()] | board.piece_bitboard[PieceType::Queen.to_index()]) 
            & board.piece_bitboard[enemy_side.to_index()];
        let enemy_rooks = (board.piece_bitboard[PieceType::Rook.to_index()] | board.piece_bitboard[PieceType::Queen.to_index()]) 
            & board.piece_bitboard[enemy_side.to_index()];
        let enemy_kings = board.piece_bitboard[PieceType::King.to_index()] & board.piece_bitboard[enemy_side.to_index()];

        let pawn_attacks = Bitboard::new(match enemy_side {
            PieceColor::White => WHITE_PAWN_MASK[self.square()].1,
            PieceColor::Black => BLACK_PAWN_MASK[self.square()].1,
        }) & enemy_pawns;
        let knight_attacks = Bitboard::new(KNIGHT_MASKS[self.square()]) & enemy_knights;
        let bishop_attacks = board.sliding_bishop_bitboard[Board::generate_magic_index(&BISHOP_MAGICS[self.square()], &board.occupied())] & enemy_bishops;
        let rook_attacks = board.sliding_rook_bitboard[Board::generate_magic_index(&ROOK_MAGICS[self.square()], &board.occupied())] & enemy_rooks;
        let king_attacks = Bitboard::new(KING_MASKS[self.square()]) & enemy_kings;

        (pawn_attacks | knight_attacks | bishop_attacks | rook_attacks | king_attacks) != Bitboard::new(0)
    }
}

/// A struct representing a chess piece.
#[derive(Debug, Clone, PartialEq)]
pub struct Piece {
    /// The type of piece.
    pub piece_type: PieceType,
    /// The color of the piece.
    pub piece_color: PieceColor
}

impl Piece {
    /// Instantiates a piece from a type and color.
    pub fn new(piece_type: PieceType, piece_color: PieceColor) -> Piece {
        Piece { piece_type, piece_color }
    }

    /// Generates a list of moves for the piece.
    pub fn generate_moves(&self, board: &mut Board, position: Position) -> Vec<Move> {
        match self.piece_type {
            PieceType::Pawn => Piece::generate_pawn_moves(board, position, self.piece_color),
            PieceType::Knight => Piece::generate_knight_moves(board, position, self.piece_color),
            PieceType::Bishop => Piece::generate_bishop_moves(board, position, self.piece_color),
            PieceType::Rook => Piece::generate_rook_moves(board, position, self.piece_color),
            PieceType::Queen => Piece::generate_bishop_moves(board, position, self.piece_color).into_iter().chain(Piece::generate_rook_moves(board, position, self.piece_color)).collect(),
            PieceType::King => Piece::generate_king_moves(board, position, self.piece_color)
        }
    }

    fn generate_pawn_moves(board: &mut Board, position: Position, piece_color: PieceColor) -> Vec<Move> {
        /*
         * TODO:
         * - Double Push (done)
         * - Check If Any Piece is Capturable (done)
         * - En Passant (done)
            * In `make_move()`, en passant square should be set to None. Then, double push should be applied.
         * - Promotion (done)
         */

        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        let direction = if piece_color == PieceColor::White { 1 } else { 0 };

        // Get movement and capture masks.
        let (mut movement, mut captures) = match piece_color {
            PieceColor::White => (Bitboard::new(WHITE_PAWN_MASK[position.square()].0), Bitboard::new(WHITE_PAWN_MASK[position.square()].1)),
            PieceColor::Black => (Bitboard::new(BLACK_PAWN_MASK[position.square()].0), Bitboard::new(BLACK_PAWN_MASK[position.square()].1))
        };
        let double_push_position = position.transform(2 * direction, 0);

        // If the pawn isn't on the correct rank, disable double push.
        if position.rank != (if piece_color == PieceColor::White { 2 } else { 7 }) {
            movement.clear_bit(double_push_position);
        }

        movement &= !board.occupied(); // Avoid moving onto pieces.
        captures &= board.piece_bitboard[!(piece_color).to_index()]; // Allow move only if an enemy piece is there.

        // Check for en passant captures.
        let mut en_passant = None;
        if let Some(ep) = board.en_passant {
            if position.transform(1 * direction, 1) == ep || position.transform(1 * direction, -1) == ep {
                en_passant = Some(ep);
            }
        }

        for r in 0..8 {
            for f in 0..8 {
                let pos = Position::new(r, f);

                if Some(pos) == en_passant {
                    moves.push(Move::new(position, pos, MoveFlags::EnPassant));
                } else if movement.get_bit(pos) || captures.get_bit(pos) {
                    if pos.rank == (if piece_color == PieceColor::White { 7 } else { 0 }) {
                        moves.push(Move::new(position, pos, MoveFlags::KnightPromotion));
                        moves.push(Move::new(position, pos, MoveFlags::BishopPromotion));
                        moves.push(Move::new(position, pos, MoveFlags::RookPromotion));
                        moves.push(Move::new(position, pos, MoveFlags::QueenPromotion));
                    } else if pos == double_push_position {
                        moves.push(Move::new(position, pos, MoveFlags::DoublePush));
                    } else {
                        moves.push(Move::new(position, pos, MoveFlags::None));
                    }
                }
            }
        }

        moves
    }

    fn generate_knight_moves(board: &mut Board, position: Position, piece_color: PieceColor) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);
        
        let mut mask = Bitboard::new(KNIGHT_MASKS[position.square()]);
        mask &= !board.piece_bitboard[piece_color.to_index()]; // Avoid capturing friendly pieces.

        for r in 0..8 {
            for f in 0..8 {
                let pos = Position::new(r, f);
                
                if mask.get_bit(pos) {
                    moves.push(Move::new(position, pos, MoveFlags::None));
                }
            }
        }

        moves
    }

    fn generate_rook_moves(board: &mut Board, position: Position, piece_color: PieceColor) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        // Retreive the mask through the magic indexing system.
        let magic = &ROOK_MAGICS[position.square()];

        let mut mask = board.sliding_rook_bitboard[Board::generate_magic_index(magic, &board.occupied())];
        mask &= !board.piece_bitboard[piece_color.to_index()]; // Avoid capturing friendly pieces.

        for r in 0..8 {
            for f in 0..8 {
                let pos = Position::new(r, f);
                
                if mask.get_bit(pos) {
                    moves.push(Move::new(position, pos, MoveFlags::None));
                }
            }
        }

        moves
    }

    fn generate_bishop_moves(board: &mut Board, position: Position, piece_color: PieceColor) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        // Retreive the mask through the magic indexing system.
        let magic = &BISHOP_MAGICS[position.square()];

        let mut mask = board.sliding_bishop_bitboard[Board::generate_magic_index(magic, &board.occupied())];
        mask &= !board.piece_bitboard[piece_color.to_index()]; // Avoid capturing friendly pieces.

        for r in 0..8 {
            for f in 0..8 {
                let pos = Position::new(r, f);
                
                if mask.get_bit(pos) {
                    moves.push(Move::new(position, pos, MoveFlags::None));
                }
            }
        }

        moves
    }

    fn generate_king_moves(board: &mut Board, position: Position, piece_color: PieceColor) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);
        
        let mut mask = Bitboard::new(KING_MASKS[position.square()]);
        mask &= !board.piece_bitboard[piece_color.to_index()]; // Avoid capturing friendly pieces.

        match board.castle_rights[piece_color.to_index()] {
            CastleRights::KingSide => {
                let can_castle = !(position.is_under_attack(board, piece_color)
                || position.transform(0, 1).is_under_attack(board, piece_color)
                || position.transform(0, 2).is_under_attack(board, piece_color));
                
                if can_castle {
                    moves.push(Move::new(position, position.transform(0, 2), MoveFlags::Castling));
                }
            },
            CastleRights::QueenSide => {
                let can_castle = !(position.is_under_attack(board, piece_color)
                || position.transform(0, -1).is_under_attack(board, piece_color)
                || position.transform(0, -2).is_under_attack(board, piece_color));

                if can_castle {
                    moves.push(Move::new(position, position.transform(0, -2), MoveFlags::Castling));
                }
            },
            CastleRights::Both => {
                let can_castle = !(position.is_under_attack(board, piece_color)
                || position.transform(0, 1).is_under_attack(board, piece_color)
                || position.transform(0, 2).is_under_attack(board, piece_color));
                
                if can_castle {
                    moves.push(Move::new(position, position.transform(0, 2), MoveFlags::Castling));
                }
                
                let can_castle = !(position.is_under_attack(board, piece_color)
                || position.transform(0, -1).is_under_attack(board, piece_color)
                || position.transform(0, -2).is_under_attack(board, piece_color));

                if can_castle {
                    moves.push(Move::new(position, position.transform(0, -2), MoveFlags::Castling));
                }
            },
            CastleRights::None => {}
        }

        for r in 0..8 {
            for f in 0..8 {
                let pos = Position::new(r, f);
                
                if mask.get_bit(pos) {
                    moves.push(Move::new(position, pos, MoveFlags::None));
                }
            }
        }

        moves
    }
}

/// An enumeration of different types of castle rights.
#[derive(Debug, Default, Clone, PartialEq)]
pub enum CastleRights {
    #[default]
    None,
    QueenSide,
    KingSide,
    Both
}

/// A structure representing a move.
#[derive(Debug)]
pub struct Move {
    /// The position the piece is moving from.
    pub initial: Position,
    /// The position the piece is moving to.
    pub end: Position,
    /// Any additional metadata with the move.
    pub metadata: MoveFlags
}

/// An enumeration of different move actions.
#[derive(Debug)]
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

impl Move {
    /// Creates a new move.
    pub fn new(initial: Position, end: Position, metadata: MoveFlags) -> Self {
        Move {
            initial,
            end,
            metadata
        }
    }
}