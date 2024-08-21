use std::{collections::BTreeSet, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not}};

use strum::EnumCount;
use super::{consts::{MagicEntry, BISHOP_MAGICS, BISHOP_TABLE_SIZE, MAX_LEGAL_MOVES, PIECE_MAP, ROOK_MAGICS, ROOK_TABLE_SIZE}, piece::*};
use colored::Colorize;

/// A type representing a 2D array representation of the chess board.
pub type ChessBoard = [[Option<Piece>; 8]; 8];
/// A type representing an array of bitboards for tracking piece/color state.
pub type PositionalBitboard = [Bitboard; PieceType::COUNT + PieceColor::COUNT];

/// A bitboard representing the presence of a specific state on the chess board.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Bitboard {
    /// The underlying u64 board.
    pub board: u64
}

impl Bitboard {
    /// Instantiates a bitboard.
    pub fn new(board: u64) -> Self {
        Bitboard { board }
    }

    /// Sets a state on the board, given a position.
    pub fn set_bit(&mut self, position: Position) {
        self.board |= 1 << position.square();
    }

    /// Clears a state on the board, given a position.
    pub fn clear_bit(&mut self, position: Position) {
        self.board &= !(1 << position.square());
    }

    /// Checks if a state is set on the board, given a position.
    pub fn get_bit(&self, position: Position) -> bool {
        self.board & (1 << position.square()) != 0
    }

    /// Renders the bitboard.
    pub fn render_bitboard(&self, position: Position) {
        for row in 0..8 {
            for col in 0..8 {
                let pos = Position::new(row, col);
                let is_set = self.get_bit(pos);
    
                print!("| ");
                if pos == position {
                    if is_set {
                        print!("{}", "X".green())
                    } else {
                        print!("{}", "X".red())
                    }
                } else if is_set {
                    print!("{}", "1".green());
                } else {
                    print!("{}", "0".red());
                }
            }
    
            println!("| ({})", row);
        }

        println!();
    }
}

impl serde::Serialize for Bitboard {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(&format!("Bitboard {{ board: {} }}", self.board))
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Bitboard { board: self.board | rhs.board }
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Bitboard { board: self.board & rhs.board }
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self {
        Bitboard { board: self.board ^ rhs.board }
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.board |= rhs.board;
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.board &= rhs.board;
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.board ^= rhs.board;
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self {
        Bitboard { board: !self.board }
    }
}

/// A structure representing the state of an entire chess board.
#[derive(Debug, Clone)]
pub struct Board {
    pub piece_bitboard: PositionalBitboard,
    pub sliding_rook_bitboard: [Bitboard; ROOK_TABLE_SIZE],
    pub sliding_bishop_bitboard: [Bitboard; BISHOP_TABLE_SIZE],
    pub castle_rights: [CastleRights; 2],
    pub side_to_move: PieceColor,
    pub en_passant: Option<Position>,
    
    pub board: ChessBoard
}

impl Board {
    pub fn default() -> Self {
        Board {
            piece_bitboard: std::array::from_fn(|_| Bitboard::default()),
            sliding_rook_bitboard: std::array::from_fn(|_| Bitboard::default()),
            sliding_bishop_bitboard: std::array::from_fn(|_| Bitboard::default()),
            castle_rights: std::array::from_fn(|_| CastleRights::default()),
            side_to_move: PieceColor::White,
            en_passant: None,
            board: ChessBoard::default()
        }
    }

    /// Initialises a chess board given a FEN string.
    /// 
    /// Returns an error if the FEN is invalid.
    pub fn new(fen: &str) -> Board {
        let mut chess_board = Board::default();
        chess_board.compute_sliding_bitboards();

        let (mut rank, mut file) = (7_u8, 0_u8);

        let tokens: Vec<&str> = fen.split(' ').collect();
        if tokens.len() < 4 {
            panic!("invalid fen: 4 tokens should be present")
        }

        let pieces = tokens[0];
        let side = tokens[1];
        let castle_rights = tokens[2];
        let en_passant = tokens[3];

        for char in pieces.chars() {
            if let Some(advance) = char.to_digit(10) {
                file += advance as u8;
                continue;
            }

            let piece_color = if char.is_uppercase() { PieceColor::White } else { PieceColor::Black };

            match char.to_ascii_lowercase() {
                '/' => {
                    rank -= 1;
                    file = 0;
                },
                'p' | 'n' | 'b' | 'r' | 'q' | 'k' => {
                    let piece_type = PIECE_MAP.get(&char.to_ascii_lowercase()).expect("").clone();
                    chess_board.board[rank as usize][file as usize] = Some(Piece::new(piece_type.clone(), piece_color.clone()));

                    chess_board.piece_bitboard[piece_type.to_index()].set_bit(Position::new(rank, file));
                    chess_board.piece_bitboard[piece_color.to_index()].set_bit(Position::new(rank, file));

                    file += 1;
                }
                _ => panic!("invalid board notation")
            }
        }

        match side.to_ascii_lowercase().as_str() {
            "w" => chess_board.side_to_move = PieceColor::White,
            "b" => chess_board.side_to_move = PieceColor::Black,
            _ => panic!("invalid side-to-move notation")
        };

        for (king_side, queen_side, color) in [("K", "Q", PieceColor::White), ("k", "q", PieceColor::Black)].iter() {
            chess_board.castle_rights[color.to_index()] = match (castle_rights.contains(king_side), castle_rights.contains(queen_side)) {
                (true, true) => CastleRights::Both,
                (true, false) => CastleRights::KingSide,
                (false, true) => CastleRights::QueenSide,
                _ => CastleRights::None,
            };
        }

        if Position::is_code_valid(en_passant) {
            chess_board.en_passant = Some(Position::from_code(en_passant));
        }
        
        chess_board
    }

    /// Returns all occupied pieces on the board.
    pub fn occupied(&self) -> Bitboard {
        self.piece_bitboard[PieceColor::White.to_index()] | self.piece_bitboard[PieceColor::Black.to_index()]
    }
        
    /// Generates all legal moves for a given piece.
    pub fn generate_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        for i in 0..8 {
            for j in 0..8 {
                if let Some(p) = self.board[i][j].clone() && p.piece_color == self.side_to_move {
                    moves.extend(p.generate_moves(self, Position::new(i as u8, j as u8)));
                }
            }
        }

        // In search/perft, filter illegal moves

        self.side_to_move = !self.side_to_move;

        moves
    }
    
    /// Applies a move to the board.
    pub fn make_move(&mut self, piece_move: &Move) {
        let initial_piece = self.board[piece_move.initial.rank as usize][piece_move.initial.file as usize].clone().expect("expected a piece on initial square");
        let end_piece = self.board[piece_move.initial.rank as usize][piece_move.initial.file as usize].clone();

        // Update the bitboards.
        self.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.initial);
        self.piece_bitboard[initial_piece.piece_color.to_index()].clear_bit(piece_move.initial);
        self.piece_bitboard[initial_piece.piece_type.to_index()].set_bit(piece_move.end);
        self.piece_bitboard[initial_piece.piece_color.to_index()].set_bit(piece_move.end);

        if let Some(ref piece) = end_piece {
            self.piece_bitboard[piece.piece_type.to_index()].clear_bit(piece_move.end);
            self.piece_bitboard[piece.piece_color.to_index()].clear_bit(piece_move.end);
        }

        // Update the mailbox board.
        self.board[piece_move.initial.rank as usize][piece_move.initial.file as usize] = None;
        self.board[piece_move.end.rank as usize][piece_move.end.file as usize] = Some(initial_piece.clone());

        let castle_rights = &mut self.castle_rights[initial_piece.piece_color.to_index()];
        if *castle_rights != CastleRights::None {
            if initial_piece.piece_type == PieceType::King {
                *castle_rights = CastleRights::None; // The right to castle has been lost if the king has already moved.
            } else if initial_piece.piece_type == PieceType::Rook {
                // The right to castle has been lost with a rook that has already moved.
                if piece_move.initial.file == 0 {
                    *castle_rights = if *castle_rights == CastleRights::QueenSide { CastleRights::None } else { CastleRights::KingSide };
                } else if piece_move.initial.file == 7 {
                    *castle_rights = if *castle_rights == CastleRights::KingSide { CastleRights::None } else { CastleRights::QueenSide };
                }
            }

            if let Some(ref piece) = end_piece && piece.piece_type == PieceType::Rook {
                // Can't castle with a dead rook
                if piece_move.end.file == 0 {
                    *castle_rights = if *castle_rights == CastleRights::QueenSide { CastleRights::None } else { CastleRights::KingSide };
                } else if piece_move.end.file == 7 {
                    *castle_rights = if *castle_rights == CastleRights::KingSide { CastleRights::None } else { CastleRights::QueenSide };
                }
            }
        }

        self.en_passant = None;

        match piece_move.metadata {
            MoveFlags::DoublePush => {
                let direction = if initial_piece.piece_color == PieceColor::White { -1 } else { 1 };
                self.en_passant = Some(piece_move.end.transform(direction, 0));
            },
            // MoveFlags::EnPassant => {},
            MoveFlags::Castling => {
                self.castle_rights[initial_piece.piece_color.to_index()] = CastleRights::None;
                let king_side = (piece_move.end.file - piece_move.initial.file) == 2;

                let old_rook_position = Position::new(
                    if initial_piece.piece_color == PieceColor::White { 0 } else { 7 }, 
                    if king_side { 7 } else { 0 }
                );

                let new_rook_position = Position::new(
                    if initial_piece.piece_color == PieceColor::White { 0 } else { 7 }, 
                    if king_side { 5 } else { 3 }
                );

                self.piece_bitboard[PieceType::Rook.to_index()].clear_bit(old_rook_position);
                self.piece_bitboard[initial_piece.piece_color.to_index()].clear_bit(old_rook_position);
                self.piece_bitboard[PieceType::Rook.to_index()].set_bit(new_rook_position);
                self.piece_bitboard[initial_piece.piece_color.to_index()].set_bit(new_rook_position);

                self.board[old_rook_position.rank as usize][old_rook_position.file as usize] = None;
                self.board[new_rook_position.rank as usize][new_rook_position.file as usize] = Some(Piece::new(PieceType::Rook, initial_piece.piece_color));
            },
            MoveFlags::KnightPromotion => {
                self.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.end);
                self.piece_bitboard[PieceType::Knight.to_index()].set_bit(piece_move.end);
                self.board[piece_move.end.rank as usize][piece_move.end.file as usize] = Some(Piece::new(PieceType::Knight, initial_piece.piece_color));
            },
            MoveFlags::BishopPromotion => {
                self.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.end);
                self.piece_bitboard[PieceType::Bishop.to_index()].set_bit(piece_move.end);
                self.board[piece_move.end.rank as usize][piece_move.end.file as usize] = Some(Piece::new(PieceType::Bishop, initial_piece.piece_color));
            },
            MoveFlags::RookPromotion => {
                self.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.end);
                self.piece_bitboard[PieceType::Rook.to_index()].set_bit(piece_move.end);
                self.board[piece_move.end.rank as usize][piece_move.end.file as usize] = Some(Piece::new(PieceType::Rook, initial_piece.piece_color));
            },
            MoveFlags::QueenPromotion => {
                self.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.end);
                self.piece_bitboard[PieceType::Queen.to_index()].set_bit(piece_move.end);
                self.board[piece_move.end.rank as usize][piece_move.end.file as usize] = Some(Piece::new(PieceType::Queen, initial_piece.piece_color));
            },
            _ => {}
        }
    }
    
    /// Precomputes the legal moves for sliding pieces.
    fn compute_sliding_bitboards(&mut self) {
        for rank in 0..8 {
            for file in 0..8 {
                let rook_blocker_bitboard = self.compute_blocker_bitboards(rank, file, PieceType::Rook);
                let bishop_blocker_bitboard = self.compute_blocker_bitboards(rank, file, PieceType::Bishop);

                for blocker_bitboard in rook_blocker_bitboard.iter() {
                    let mut legal_moves = Bitboard::new(0);
                    let magic = &ROOK_MAGICS[rank * 8 + file];
                    let index = Board::generate_magic_index(magic, blocker_bitboard);

                    Board::generate_rook_moves(rank as u8, file as u8, &mut legal_moves, blocker_bitboard);
                    self.sliding_rook_bitboard[index] = legal_moves;
                }

                for blocker_bitboard in bishop_blocker_bitboard.iter() {       
                    let mut legal_moves = Bitboard::new(0);
                    let magic = &BISHOP_MAGICS[rank * 8 + file];
                    let index = Board::generate_magic_index(magic, blocker_bitboard);

                    legal_moves |= Board::generate_bishop_moves(rank as i8, file as i8, 1, 1, blocker_bitboard);
                    legal_moves |= Board::generate_bishop_moves(rank as i8, file as i8, 1, -1, blocker_bitboard);
                    legal_moves |= Board::generate_bishop_moves(rank as i8, file as i8, -1, 1, blocker_bitboard);
                    legal_moves |= Board::generate_bishop_moves(rank as i8, file as i8, -1, -1, blocker_bitboard);

                    self.sliding_bishop_bitboard[index] = legal_moves;
                }
            }
        }
    }

    /// Precomputes the blocker bitboards for a slidng piece.
    fn compute_blocker_bitboards(&mut self, rank: usize, file: usize, piece_type: PieceType) -> Vec<Bitboard> {
        let attack_bitboard = match piece_type {
            PieceType::Rook => Bitboard::new(ROOK_MAGICS[rank * 8 + file].mask),
            PieceType::Bishop => Bitboard::new(BISHOP_MAGICS[rank * 8 + file].mask),
            _ => panic!("blocker bitboard cannot be computed for non-sliding piece")
        };

        let mut move_square_indices = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                if attack_bitboard.get_bit(Position::new(rank, file)) {
                    move_square_indices.push(rank * 8 + file);
                }
            }
        }

        let num_combos = 1 << move_square_indices.len();
        let mut blocker_bitboards: Vec<Bitboard> = Vec::with_capacity(num_combos);

        for pattern_idx in 0..num_combos {
            blocker_bitboards.push(Bitboard::default());

            for (bit_idx, move_square_idx) in move_square_indices.iter().enumerate() {
                let bit: u64 = ((pattern_idx >> bit_idx) & 1) as u64;
                blocker_bitboards[pattern_idx].board |= bit << move_square_idx;
            }
        }

        blocker_bitboards
    }

    /// Generates bishop moves in a certain direction.
    fn generate_rook_moves(rank: u8, file: u8, legal_moves: &mut Bitboard, blocker_bitboard: &Bitboard) {
        for rank2 in rank..8 {
            let current_position = Position::new(rank2, file);
            if current_position == Position::new(rank, file) { continue; }

            legal_moves.set_bit(current_position);
            if blocker_bitboard.get_bit(current_position) {
                break;
            }
        }

        for rank2 in (0..rank).rev() {
            let current_position = Position::new(rank2, file);
            if current_position == Position::new(rank, file) { continue; }

            legal_moves.set_bit(current_position);
            if blocker_bitboard.get_bit(current_position) {
                break;
            }
        }

        for file2 in file..8 {
            let current_position = Position::new(rank, file2);
            if current_position == Position::new(rank, file) { continue; }

            legal_moves.set_bit(current_position);
            if blocker_bitboard.get_bit(current_position) {
                break;
            }
        }

        for file2 in (0..file).rev() {
            let current_position = Position::new(rank, file2);
            if current_position == Position::new(rank, file) { continue; }

            legal_moves.set_bit(current_position);
            if blocker_bitboard.get_bit(current_position) {
                break;
            }
        }
    }

    /// Generates bishop moves in a certain direction.
    fn generate_bishop_moves(rank: i8, file: i8, rank_offset: i8, file_offset: i8, blocker_bitboard: &Bitboard) -> Bitboard {
        let mut legal_moves = Bitboard::new(0);
    
        for offset in 1..7 {
            if !Position::is_valid((rank + offset * rank_offset) as u8, (file + offset * file_offset) as u8) {
                break;
            }

            let current_position = Position::new((rank + offset * rank_offset) as u8, (file + offset * file_offset) as u8);
            
            if current_position == Position::new(rank as u8, file as u8) { continue; }
            legal_moves.set_bit(current_position);
            if blocker_bitboard.get_bit(current_position) { break; }
        }
    
        legal_moves
    }

    /// Generates a magic index given a magic entry and a blocker bitboard.
    pub fn generate_magic_index(magic: &MagicEntry, blockers: &Bitboard) -> usize {
        let blockers = blockers.board & magic.mask;
        let hash = blockers.wrapping_mul(magic.magic);
        let index = (hash >> magic.shift) as usize;
        magic.offset as usize + index
    }
}