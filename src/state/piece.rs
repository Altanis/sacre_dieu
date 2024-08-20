use std::ops::Not;

use super::{board::{AttackBitboard, Board, PositionalBitboard}, consts::{BISHOP_MAGICS, MAX_LEGAL_MOVES, ROOK_MAGICS}};

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
#[derive(Debug, Default, Clone, PartialEq, strum_macros::EnumCount)]
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
    pub fn square(&self) -> u8 {
        self.rank * 8 + self.file
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
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        match self.piece_type {
            PieceType::Pawn => {
                let enemy_bitboard = &board.positional_bitboard[(!self.piece_color.clone()).to_index()];
                
                let direction = if self.piece_color == PieceColor::White { 1 } else { -1 };

                let can_double_push = if self.piece_color == PieceColor::White { position.rank == 2 } else { position.rank == 7 };
                let (can_capture_left, can_capture_right) = (
                    enemy_bitboard.get_bit(position.transform(direction * 1, -1)),
                    enemy_bitboard.get_bit(position.transform(direction * 1, 1)),
                );

                moves.push(Move::new(position, position.transform(direction * 1, 0), MoveFlags::None));

                if can_double_push {
                    moves.push(Move::new(position, position.transform(direction * 2, 0), MoveFlags::DoublePush));
                }

                if can_capture_left {
                    moves.push(Move::new(position, position.transform(direction * 1, -1), MoveFlags::None));
                }

                if can_capture_right {
                    moves.push(Move::new(position, position.transform(direction * 1, 1), MoveFlags::None));
                }
            },
            PieceType::Knight => {
                let mask = &board.attack_bitboard[self.piece_type.to_index()][position.square() as usize];
                for rank in 0..8 {
                    for file in 0..8 {
                        let new_pos = Position::new(rank, file);

                        if mask.get_bit(new_pos) {
                            moves.push(Move::new(position, new_pos, MoveFlags::None));
                        }
                    }
                }
            },
            PieceType::Bishop => {
                let magic = &BISHOP_MAGICS[position.square() as usize];
                let enemy_bitboard = &board.positional_bitboard[(!self.piece_color.clone()).to_index()];
                let mask = &board.sliding_bishop_bitboard[Board::generate_magic_index(magic, enemy_bitboard)];

                for rank in 0..8 {
                    for file in 0..8 {
                        let new_pos = Position::new(rank, file);

                        if mask.get_bit(new_pos) {
                            moves.push(Move::new(position, new_pos, MoveFlags::None));
                        }
                    }
                }
            },
            PieceType::Rook => {
                let magic = &ROOK_MAGICS[position.square() as usize];
                let enemy_bitboard = &board.positional_bitboard[(!self.piece_color.clone()).to_index()];
                let mask = &board.sliding_rook_bitboard[Board::generate_magic_index(magic, enemy_bitboard)];

                for rank in 0..8 {
                    for file in 0..8 {
                        let new_pos = Position::new(rank, file);

                        if mask.get_bit(new_pos) {
                            moves.push(Move::new(position, new_pos, MoveFlags::None));
                        }
                    }
                }
            },
            PieceType::Queen => {
                {
                    let magic = &BISHOP_MAGICS[position.square() as usize];
                    let enemy_bitboard = &board.positional_bitboard[(!self.piece_color.clone()).to_index()];
                    let mask = &board.sliding_bishop_bitboard[Board::generate_magic_index(magic, enemy_bitboard)];
    
                    for rank in 0..8 {
                        for file in 0..8 {
                            let new_pos = Position::new(rank, file);
    
                            if mask.get_bit(new_pos) {
                                moves.push(Move::new(position, new_pos, MoveFlags::None));
                            }
                        }
                    }    
                }

                {
                    let magic = &ROOK_MAGICS[position.square() as usize];
                    let enemy_bitboard = &board.positional_bitboard[(!self.piece_color.clone()).to_index()];
                    let mask = &board.sliding_rook_bitboard[Board::generate_magic_index(magic, enemy_bitboard)];
    
                    for rank in 0..8 {
                        for file in 0..8 {
                            let new_pos = Position::new(rank, file);
    
                            if mask.get_bit(new_pos) {
                                moves.push(Move::new(position, new_pos, MoveFlags::None));
                            }
                        }
                    }
                }
            },
            PieceType::King => {
                let mask = &board.attack_bitboard[self.piece_type.to_index()][position.square() as usize];
                for rank in 0..8 {
                    for file in 0..8 {
                        let new_pos = Position::new(rank, file);

                        if mask.get_bit(new_pos) {
                            moves.push(Move::new(position, new_pos, MoveFlags::None));
                        }
                    }
                }
            }
        }

        moves
    }
}

/// An enumeration of different types of castle rights.
#[derive(Debug, Default, PartialEq)]
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
    None             = 0,
    Castling         = 1 << 0,
    DoublePush       = 1 << 1,
    EnPassant        = 1 << 2,
    KnightPromotion  = 1 << 3,
    BishopPromotion  = 1 << 4,
    RookPromotion    = 1 << 5,
    QueenPromotion   = 1 << 6
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