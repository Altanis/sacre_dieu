use std::ops::Not;

use arrayvec::ArrayVec;

use super::{board::{Bitboard, Board}, consts::{get_bishop_mask, get_rook_mask, BISHOP_MAGICS, BISHOP_VALUE, BLACK_PAWN_MASK, KING_MASKS, KING_VALUE, KNIGHT_MASKS, KNIGHT_VALUE, MAX_LEGAL_MOVES, PAWN_VALUE, QUEEN_VALUE, ROOK_MAGICS, ROOK_VALUE, WHITE_PAWN_MASK}, piece_move::{Move, MoveArray, MoveFlags}, zobrist::ZOBRIST_PIECE_KEYS};

/// An enum representing the type of chess piece.
#[derive(Debug, Clone, Copy, PartialEq, strum_macros::EnumCount, strum_macros::EnumIter)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
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

    /// Converts the piece type to a weighted value.
    pub fn get_value(&self) -> i32 {
        match self {
            PieceType::Pawn => PAWN_VALUE,
            PieceType::Knight => KNIGHT_VALUE,
            PieceType::Bishop => BISHOP_VALUE,
            PieceType::Rook => ROOK_VALUE,
            PieceType::Queen => QUEEN_VALUE,
            PieceType::King => KING_VALUE
        }
    }
}

/// An enum representing the color of a chess piece.
#[derive(Debug, Default, Clone, Copy, PartialEq, strum_macros::EnumCount)]
pub enum PieceColor {
    #[default]
    White,
    Black
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
    pub fn new(rank: u8, file: u8) -> Option<Tile> {
        if rank > 7 || file > 7 {
            // paniep!("invalid position {} {}", rank, file);
            return None;
        }

        Some(Tile { rank, file })
    }

    /// Instantiates a new tile from an index.
    pub fn from_index(index: u8) -> Option<Tile> {
        if index > 63 {
            return None;
        }

        Some(Tile { rank: index / 8, file: index % 8 })
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
    pub fn transform(&self, offset_rank: i8, offset_file: i8) -> Option<Self> {
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

        Tile::new(rank, file).unwrap()
    }

    /// Whether or not a code is valid.
    pub fn is_code_valid(code: &str) -> bool {
        code.len() == 2 && code.chars().nth(1).and_then(|c| c.to_digit(10)).is_some()
    }

    /// The attackers of a tile, irrespective of color.
    pub fn attackers(&self, board: &Board, occupied: Bitboard) -> Bitboard {
        let white_pawns = board.colored_piece(PieceType::Pawn, PieceColor::White);
        let black_pawns = board.colored_piece(PieceType::Pawn, PieceColor::Black);

        let knights = board.piece(PieceType::Knight);
        let bishops = board.piece(PieceType::Bishop) | board.piece(PieceType::Queen);
        let rooks = board.piece(PieceType::Rook) | board.piece(PieceType::Queen);
        let kings = board.piece(PieceType::King);

        let pawn_attacks = (Bitboard::new(BLACK_PAWN_MASK[self.index()].1) & white_pawns) | (Bitboard::new(WHITE_PAWN_MASK[self.index()].1) & black_pawns);
        let knight_attacks = Bitboard::new(KNIGHT_MASKS[self.index()]) & knights;
        let bishop_attacks = get_bishop_mask(Board::generate_magic_index(&BISHOP_MAGICS[self.index()], &occupied)) & bishops;
        let rook_attacks = get_rook_mask(Board::generate_magic_index(&ROOK_MAGICS[self.index()], &occupied)) & rooks;
        let king_attacks = Bitboard::new(KING_MASKS[self.index()]) & kings;

        pawn_attacks | knight_attacks | bishop_attacks | rook_attacks | king_attacks
    }

    /// The attackers for the tile of a specific color.
    pub fn colored_attackers(&self, board: &Board, enemy_side: PieceColor) -> Bitboard {
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

        pawn_attacks | knight_attacks | bishop_attacks | rook_attacks | king_attacks
    }

    /// Whether or not the position is under attack from a specific side.
    pub fn is_under_attack(&self, board: &Board, enemy_side: PieceColor) -> bool {
        self.colored_attackers(board, enemy_side) != Bitboard::ZERO
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

    /// Generates a XOR-able 64 bit number to toggle a state in a Zobrist key encoded position.
    pub fn zobrist_key(&self, tile_index: usize) -> u64 {
        let piece_index = self.piece_color.to_index() + self.piece_type.to_index();
        ZOBRIST_PIECE_KEYS[piece_index][tile_index]
    }

    /// Generates a list of moves for the piece.
    pub fn generate_moves(&self, board: &Board, tile_start: Tile, qsearch: bool, moves: &mut MoveArray) {
        match self.piece_type {
            PieceType::Pawn => Piece::generate_pawn_moves(board, tile_start, self.piece_color, qsearch, moves),
            PieceType::Knight => Piece::generate_knight_moves(board, tile_start, self.piece_color, qsearch, moves),
            PieceType::Bishop => Piece::generate_bishop_moves(board, tile_start, self.piece_color, qsearch, moves),
            PieceType::Rook => Piece::generate_rook_moves(board, tile_start, self.piece_color, qsearch, moves),
            PieceType::Queen => {
                Piece::generate_bishop_moves(board, tile_start, self.piece_color, qsearch, moves);
                Piece::generate_rook_moves(board, tile_start, self.piece_color, qsearch, moves);
            },
            PieceType::King => Piece::generate_king_moves(board, tile_start, self.piece_color, qsearch, moves)
        };
    }

    fn generate_pawn_moves(board: &Board, tile_start: Tile, piece_color: PieceColor, qsearch: bool, moves: &mut MoveArray) {
        let direction = if piece_color == PieceColor::White { 1 } else { -1 };

        // Get movement and capture masks.
        let (mut movement, mut captures) = match piece_color {
            PieceColor::White => (Bitboard::new(WHITE_PAWN_MASK[tile_start.index()].0), Bitboard::new(WHITE_PAWN_MASK[tile_start.index()].1)),
            PieceColor::Black => (Bitboard::new(BLACK_PAWN_MASK[tile_start.index()].0), Bitboard::new(BLACK_PAWN_MASK[tile_start.index()].1))
        };

        let single_push_tile = tile_start.transform(1 * direction, 0);
        let double_push_tile = tile_start.transform(2 * direction, 0);

        if let Some(dpt) = double_push_tile {
            let start_rank = if piece_color == PieceColor::White { 1 } else { 6 };
            let is_double_push_invalid = tile_start.rank != start_rank || board.occupied().get_bit(dpt);

            if is_double_push_invalid {
                movement.clear_bit(dpt);
            }
        }

        if let Some(spt) = single_push_tile {
            if board.occupied().get_bit(spt) {
                movement.clear_bit(spt);

                if let Some(dpt) = double_push_tile {
                    movement.clear_bit(dpt);
                }
            }
        }

        // movement &= !board.occupied(); // Avoid moving onto pieces.
        captures &= board.color(!piece_color); // Allow move only if an enemy piece is there.

        // Check for en passant captures.
        let mut en_passant = None;
        if let Some(ep) = board.en_passant {
            if tile_start.transform(1 * direction, 1) == Some(ep) {
                en_passant = Some(ep);
                captures.set_bit(ep);
            }

            if tile_start.transform(1 * direction, -1) == Some(ep) {
                en_passant = Some(ep);
                captures.set_bit(ep);
            }
        }

        let mut mask = movement | captures;
        while mask.board != 0 {
            let tile_end = mask.pop_lsb();

            if Some(tile_end) == en_passant {
                moves.push(Move::new(tile_start, tile_end, MoveFlags::EnPassant));
            } else if tile_end.rank == (if piece_color == PieceColor::White { 7 } else { 0 }) {
                moves.push(Move::new(tile_start, tile_end, MoveFlags::KnightPromotion));
                moves.push(Move::new(tile_start, tile_end, MoveFlags::BishopPromotion));
                moves.push(Move::new(tile_start, tile_end, MoveFlags::RookPromotion));
                moves.push(Move::new(tile_start, tile_end, MoveFlags::QueenPromotion));
            } else {
                if qsearch && board.board[tile_end.index()].is_none() {
                    continue;
                }
    
                if Some(tile_end) == double_push_tile {
                    moves.push(Move::new(tile_start, tile_end, MoveFlags::DoublePush));
                } else {
                    moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
                }
            }
        }
    }

    fn generate_knight_moves(board: &Board, tile_start: Tile, piece_color: PieceColor, qsearch: bool, moves: &mut MoveArray) {
        let mut mask = Bitboard::new(KNIGHT_MASKS[tile_start.index()]);
        mask &= !board.color(piece_color); // Avoid capturing friendly pieces.

        let mut mask_clone = mask;
        while mask_clone.board != 0 {
            let tile_end = mask_clone.pop_lsb();
            if qsearch && board.board[tile_end.index()].is_none() {
                continue;
            }

            moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
        }
    }

    fn generate_rook_moves(board: &Board, tile_start: Tile, piece_color: PieceColor, qsearch: bool, moves: &mut MoveArray) {
        // Retreive the mask through the magic indexing system.
        let magic = &ROOK_MAGICS[tile_start.index()];

        let mut mask = get_rook_mask(Board::generate_magic_index(magic, &board.occupied()));
        mask &= !board.color(piece_color); // Avoid capturing friendly pieces.

        let mut mask_clone = mask;
        while mask_clone.board != 0 {
            let tile_end = mask_clone.pop_lsb();
            if qsearch && board.board[tile_end.index()].is_none() {
                continue;
            }

            moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
        }
    }

    fn generate_bishop_moves(board: &Board, tile_start: Tile, piece_color: PieceColor, qsearch: bool, moves: &mut MoveArray) {
        // Retreive the mask through the magic indexing system.
        let magic = &BISHOP_MAGICS[tile_start.index()];

        let mut mask = get_bishop_mask(Board::generate_magic_index(magic, &board.occupied()));
        mask &= !board.color(piece_color); // Avoid capturing friendly pieces.

        let mut mask_clone = mask;
        while mask_clone.board != 0 {
            let tile_end = mask_clone.pop_lsb();
            if qsearch && board.board[tile_end.index()].is_none() {
                continue;
            }

            moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
        }
    }

    fn generate_king_moves(board: &Board, tile_start: Tile, piece_color: PieceColor, qsearch: bool, moves: &mut MoveArray) {
        let mut mask = Bitboard::new(KING_MASKS[tile_start.index()]);
        mask &= !board.color(piece_color); // Avoid capturing friendly pieces.

        let occupied = board.occupied();

        match board.castle_rights[piece_color.to_index()] {
            CastleRights::KingSide => {
                let first_tile = tile_start.transform(0, 1);
                let second_tile = tile_start.transform(0, 2);

                if let Some(first_tile) = first_tile && let Some(second_tile) = second_tile {
                    let can_castle = !(tile_start.is_under_attack(board, !piece_color)
                    || first_tile.is_under_attack(board, !piece_color) || occupied.get_bit(first_tile)
                    || second_tile.is_under_attack(board, !piece_color) || occupied.get_bit(second_tile));
    
                    if can_castle && !qsearch {
                        moves.push(Move::new(tile_start, second_tile, MoveFlags::Castling));
                    }
                }
            },
            CastleRights::QueenSide => {
                let first_tile = tile_start.transform(0, -1);
                let second_tile = tile_start.transform(0, -2);
                let third_tile = tile_start.transform(0, -3);

                if let Some(first_tile) = first_tile && let Some(second_tile) = second_tile && let Some(third_tile) = third_tile {
                    let can_castle = !(tile_start.is_under_attack(board, !piece_color)
                    || first_tile.is_under_attack(board, !piece_color) || occupied.get_bit(first_tile)
                    || second_tile.is_under_attack(board, !piece_color) || occupied.get_bit(second_tile)
                    || occupied.get_bit(third_tile));
    
                    if can_castle && !qsearch {
                        moves.push(Move::new(tile_start, second_tile, MoveFlags::Castling));
                    }
                }
            },
            CastleRights::Both => {
                {
                    let first_tile = tile_start.transform(0, 1);
                    let second_tile = tile_start.transform(0, 2);
    
                    if let Some(first_tile) = first_tile && let Some(second_tile) = second_tile {
                        let can_castle = !(tile_start.is_under_attack(board, !piece_color)
                        || first_tile.is_under_attack(board, !piece_color) || occupied.get_bit(first_tile)
                        || second_tile.is_under_attack(board, !piece_color) || occupied.get_bit(second_tile));
        
                        if can_castle && !qsearch {
                            moves.push(Move::new(tile_start, second_tile, MoveFlags::Castling));
                        }
                    }
                }

                {
                    let first_tile = tile_start.transform(0, -1);
                    let second_tile = tile_start.transform(0, -2);
                    let third_tile = tile_start.transform(0, -3);
    
                    if let Some(first_tile) = first_tile && let Some(second_tile) = second_tile && let Some(third_tile) = third_tile {
                        let can_castle = !(tile_start.is_under_attack(board, !piece_color)
                        || first_tile.is_under_attack(board, !piece_color) || occupied.get_bit(first_tile)
                        || second_tile.is_under_attack(board, !piece_color) || occupied.get_bit(second_tile)
                        || occupied.get_bit(third_tile));
        
                        if can_castle && !qsearch {
                            moves.push(Move::new(tile_start, second_tile, MoveFlags::Castling));
                        }
                    }
                }
            },
            CastleRights::None => {}
        }

        let mut mask_clone = mask;
        while mask_clone.board != 0 {
            let tile_end = mask_clone.pop_lsb();
            if qsearch && board.board[tile_end.index()].is_none() {
                continue;
            }

            moves.push(Move::new(tile_start, tile_end, MoveFlags::None));
        }
    }
}

/// An enumeration of different types of castle rights.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum CastleRights {
    #[default]
    None,
    QueenSide,
    KingSide,
    Both
}