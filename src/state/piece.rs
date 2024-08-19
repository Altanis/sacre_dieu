/// A type representing a 2D array representation of the chess board.
pub type ChessBoard = [[Option<Piece>; 8]; 8];

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
    /// 
    /// Returns an error if the position is out of bounds.
    pub fn new(rank: u8, file: u8) -> Result<Position, ()> {
        if rank > 7 || file > 7 {
            return Err(());
        }

        Ok(Position { rank, file })
    }

    /// Converts a position to a square code.
    pub fn to_code(&self) -> String {
        format!("{}{}", (self.file + b'a') as char, self.rank + 1)
    }

    /// Instantiates a new position from a square code.
    /// 
    /// Returns an error if the code is invalid or the position is out of bounds.
    pub fn from_code(code: &str) -> Result<Position, ()> {
        if code.len() != 2 {
            return Err(());
        }

        let rank = code.chars().nth(1).ok_or(())?.to_digit(10).ok_or(())? as u8 - 1;
        let file = code.chars().nth(0).expect("Code doesn't have file") as u8 - b'a';

        Position::new(rank, file)
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
pub struct Move {
    /// The position the piece is moving from.
    pub from: Position,
    /// The position the piece is moving to.
    pub to: Position,
    /// Any additional metadata with the move.
    pub metadata: MoveFlags
}

/// An enumeration of different move actions.
pub enum MoveFlags {
    None             = 0,
    Capture          = 1,
    Castling         = 2,
    DoublePush       = 4,
    EnPassant        = 8,
    KnightPromotion  = 16,
    BishopPromotion  = 32,
    RookPromotion    = 64,
    QueenPromotion   = 128
}