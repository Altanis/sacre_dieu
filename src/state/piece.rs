use std::ops::Not;

use super::{board::{Bitboard, Board, PositionalBitboard}, consts::{get_rook_mask, get_bishop_mask, BISHOP_MAGICS, BLACK_PAWN_MASK, KING_MASKS, KNIGHT_MASKS, MAX_LEGAL_MOVES, ROOK_MAGICS, WHITE_PAWN_MASK}};

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

/// A square tile in chess.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
    pub rank: u8,
    pub file: u8
}

impl Tile {
    /// Instantiates a new tile.
    pub fn new(rank: u8, file: u8) -> Tile {
        if rank > 7 || file > 7 {
            panic!("invalid position {} {}", rank, file);
        }

        Tile { rank, file }
    }

    /// Whether or not a tile is valid.
    pub fn is_valid(rank: u8, file: u8) -> bool {
        !(rank > 7 || file > 7)
    }

    /// Returns the index of the tile.
    pub fn index(&self) -> usize {
        (self.rank * 8 + self.file) as usize
    }

    /// Returns a transformed tile.
    pub fn transform(&self, offset_rank: i8, offset_file: i8) -> Self {
        // dbg!(&self, offset_rank, offset_file);
        Tile::new((self.rank as i8 + offset_rank) as u8, (self.file as i8 + offset_file) as u8)
    }

    /// Converts a tile to a square code.
    pub fn get_code(&self) -> String {
        format!("{}{}", (self.file + b'a') as char, self.rank + 1)
    }

    /// Instantiates a new tile from a square code.
    pub fn from_code(code: &str) -> Tile {
        if code.len() != 2 {
            panic!("invalid code");
        }

        let rank = code.chars().nth(1).expect("Code doesn't have rank").to_digit(10).expect("Rank can't be converted to number") as u8 - 1;
        let file = code.chars().nth(0).expect("Code doesn't have file") as u8 - b'a';

        Tile::new(rank, file)
    }

    /// Whether or not a code is valid.
    pub fn is_code_valid(code: &str) -> bool {
        code.len() == 2 && code.chars().nth(1).and_then(|c| c.to_digit(10)).is_some()
    }    

    /// Whether or not the position is under attack from a specific side.
    pub fn is_under_attack(&self, board: &Board, enemy_side: PieceColor) -> bool {
        let enemy_pawns = board.colored_piece(PieceType::Pawn, enemy_side);
        let enemy_knights = board.colored_piece(PieceType::Knight, enemy_side);
        let enemy_bishops = board.colored_piece(PieceType::Bishop, enemy_side) | board.colored_piece(PieceType::Queen, enemy_side);
        let enemy_rooks = board.colored_piece(PieceType::Rook, enemy_side) | board.colored_piece(PieceType::Queen, enemy_side);
        let enemy_kings = board.colored_piece(PieceType::King, enemy_side);

        let pawn_attacks = Bitboard::new(match enemy_side {
            PieceColor::White => BLACK_PAWN_MASK[self.index()].1,
            PieceColor::Black => WHITE_PAWN_MASK[self.index()].1,
        }) & enemy_pawns;
        let knight_attacks = Bitboard::new(KNIGHT_MASKS[self.index()]) & enemy_knights;
        let bishop_attacks = get_bishop_mask(Board::generate_magic_index(&BISHOP_MAGICS[self.index()], &board.occupied())) & enemy_bishops;
        let rook_attacks = get_rook_mask(Board::generate_magic_index(&ROOK_MAGICS[self.index()], &board.occupied())) & enemy_rooks;
        let king_attacks = Bitboard::new(KING_MASKS[self.index()]) & enemy_kings;

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
    pub fn generate_moves(&self, board: &Board, tile_start: Tile) -> Vec<Move> {
        match self.piece_type {
            PieceType::Pawn => Piece::generate_pawn_moves(board, tile_start, self.piece_color),
            PieceType::Knight => Piece::generate_knight_moves(board, tile_start, self.piece_color),
            PieceType::Bishop => Piece::generate_bishop_moves(board, tile_start, self.piece_color),
            PieceType::Rook => Piece::generate_rook_moves(board, tile_start, self.piece_color),
            PieceType::Queen => Piece::generate_bishop_moves(board, tile_start, self.piece_color).into_iter().chain(Piece::generate_rook_moves(board, tile_start, self.piece_color)).collect(),
            PieceType::King => Piece::generate_king_moves(board, tile_start, self.piece_color)
        }
    }

    fn generate_pawn_moves(board: &Board, tile_start: Tile, piece_color: PieceColor) -> Vec<Move> {
        /*
         * TODO:
         * - Double Push (done)
         * - Check If Any Piece is Capturable (done)
         * - En Passant (done)
            * In `make_move()`, en passant square should be set to None. Then, double push should be applied.
         * - Promotion (done)
         */

        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        let direction = if piece_color == PieceColor::White { 1 } else { -1 };

        // Get movement and capture masks.
        let (mut movement, mut captures) = match piece_color {
            PieceColor::White => (Bitboard::new(WHITE_PAWN_MASK[tile_start.index()].0), Bitboard::new(WHITE_PAWN_MASK[tile_start.index()].1)),
            PieceColor::Black => (Bitboard::new(BLACK_PAWN_MASK[tile_start.index()].0), Bitboard::new(BLACK_PAWN_MASK[tile_start.index()].1))
        };
        let double_push_tile = tile_start.transform(2 * direction, 0);

        if tile_start.rank != (if piece_color == PieceColor::White { 1 } else { 6 }) || board.occupied().get_bit(double_push_tile) {
            movement.clear_bit(double_push_tile);
        }

        if board.occupied().get_bit(tile_start.transform(1 * direction, 0)) {
            movement.clear_bit(tile_start.transform(1 * direction, 0));
            movement.clear_bit(double_push_tile);
        }

        // movement &= !board.occupied(); // Avoid moving onto pieces.
        captures &= board.color(!piece_color); // Allow move only if an enemy piece is there.

        // Check for en passant captures.
        let mut en_passant = None;
        if let Some(ep) = board.en_passant {
            if tile_start.file != 7 && tile_start.transform(1 * direction, 1) == ep {
                en_passant = Some(ep);
            }

            if tile_start.file != 0 && tile_start.transform(1 * direction, -1) == ep {
                en_passant = Some(ep);
            }
        }

        for r in 0..8 {
            for f in 0..8 {
                let tile_end = Tile::new(r, f);

                if Some(tile_end) == en_passant {
                    moves.push(Move::new(tile_start, tile_end, MoveFlags::EnPassant));
                } else if movement.get_bit(tile_end) || captures.get_bit(tile_end) {
                    if tile_end.rank == (if piece_color == PieceColor::White { 7 } else { 0 }) {
                        moves.push(Move::new(tile_start, tile_end, MoveFlags::KnightPromotion));
                        moves.push(Move::new(tile_start, tile_end, MoveFlags::BishopPromotion));
                        moves.push(Move::new(tile_start, tile_end, MoveFlags::RookPromotion));
                        moves.push(Move::new(tile_start, tile_end, MoveFlags::QueenPromotion));
                    } else if tile_end == double_push_tile {
                        moves.push(Move::new(tile_start, tile_end, MoveFlags::DoublePush));
                    } else {
                        moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
                    }
                }
            }
        }

        moves
    }

    fn generate_knight_moves(board: &Board, tile_start: Tile, piece_color: PieceColor) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);
        
        let mut mask = Bitboard::new(KNIGHT_MASKS[tile_start.index()]);
        mask &= !board.color(piece_color); // Avoid capturing friendly pieces.

        for r in 0..8 {
            for f in 0..8 {
                let tile_end = Tile::new(r, f);

                if mask.get_bit(tile_end) {
                    moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
                }
            }
        }

        moves
    }

    fn generate_rook_moves(board: &Board, tile_start: Tile, piece_color: PieceColor) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        // Retreive the mask through the magic indexing system.
        let magic = &ROOK_MAGICS[tile_start.index()];

        let mut mask = get_rook_mask(Board::generate_magic_index(magic, &board.occupied()));
        mask &= !board.color(piece_color); // Avoid capturing friendly pieces.

        for r in 0..8 {
            for f in 0..8 {
                let tile_end = Tile::new(r, f);

                if mask.get_bit(tile_end) {
                    moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
                }
            }
        }

        moves
    }

    fn generate_bishop_moves(board: &Board, tile_start: Tile, piece_color: PieceColor) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        // Retreive the mask through the magic indexing system.
        let magic = &BISHOP_MAGICS[tile_start.index()];

        let mut mask = get_bishop_mask(Board::generate_magic_index(magic, &board.occupied()));
        mask &= !board.color(piece_color); // Avoid capturing friendly pieces.

        for r in 0..8 {
            for f in 0..8 {
                let tile_end = Tile::new(r, f);

                if mask.get_bit(tile_end) {
                    moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
                }
            }
        }

        moves
    }

    fn generate_king_moves(board: &Board, tile_start: Tile, piece_color: PieceColor) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);
        
        let mut mask = Bitboard::new(KING_MASKS[tile_start.index()]);
        mask &= !board.color(piece_color); // Avoid capturing friendly pieces.

        let occupied = board.occupied();

        match board.castle_rights[piece_color.to_index()] {
            CastleRights::KingSide => {
                let can_castle = !(tile_start.is_under_attack(board, !piece_color)
                || tile_start.transform(0, 1).is_under_attack(board, !piece_color) || occupied.get_bit(tile_start.transform(0, 1))
                || tile_start.transform(0, 2).is_under_attack(board, !piece_color) || occupied.get_bit(tile_start.transform(0, 2)));

                if can_castle {
                    moves.push(Move::new(tile_start, tile_start.transform(0, 2), MoveFlags::Castling));
                }
            },
            CastleRights::QueenSide => {
                let can_castle = !(tile_start.is_under_attack(board, !piece_color)
                || tile_start.transform(0, -1).is_under_attack(board, !piece_color) || occupied.get_bit(tile_start.transform(0, -1))
                || tile_start.transform(0, -2).is_under_attack(board, !piece_color) || occupied.get_bit(tile_start.transform(0, -2)));

                if can_castle {
                    moves.push(Move::new(tile_start, tile_start.transform(0, -2), MoveFlags::Castling));
                }
            },
            CastleRights::Both => {
                let can_castle = !(tile_start.is_under_attack(board, !piece_color)
                || tile_start.transform(0, 1).is_under_attack(board, !piece_color) || occupied.get_bit(tile_start.transform(0, 1))
                || tile_start.transform(0, 2).is_under_attack(board, !piece_color) || occupied.get_bit(tile_start.transform(0, 2)));

                if can_castle {
                    moves.push(Move::new(tile_start, tile_start.transform(0, 2), MoveFlags::Castling));
                }
                
                let can_castle = !(tile_start.is_under_attack(board, !piece_color)
                || tile_start.transform(0, -1).is_under_attack(board, !piece_color) || occupied.get_bit(tile_start.transform(0, -1))
                || tile_start.transform(0, -2).is_under_attack(board, !piece_color) || occupied.get_bit(tile_start.transform(0, -2)));

                if can_castle {
                    moves.push(Move::new(tile_start, tile_start.transform(0, -2), MoveFlags::Castling));
                }
            },
            CastleRights::None => {}
        }

        for r in 0..8 {
            for f in 0..8 {
                let tile_end = Tile::new(r, f);

                if mask.get_bit(tile_end) {
                    moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
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
    /// The tile the piece is moving from.
    pub initial: Tile,
    /// The tile the piece is moving to.
    pub end: Tile,
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
    pub fn new(initial: Tile, end: Tile, metadata: MoveFlags) -> Self {
        Move {
            initial,
            end,
            metadata
        }
    }
}