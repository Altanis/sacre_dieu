use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use strum::EnumCount;
use super::{consts::{MagicEntry, MAX_LEGAL_MOVES, PIECE_MAP}, piece::*};
use colored::Colorize;

/// A type representing a 2D array representation of the chess board.
pub type ChessBoard = [[Option<Piece>; 8]; 8];
/// A type representing an array of bitboards for tracking piece/color state.
pub type PositionalBitboard = [Bitboard; PieceType::COUNT + PieceColor::COUNT];

/// A bitboard representing the presence of a specific state on the chess board.
#[derive(Debug, Default, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
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
    
                print!( "| ");
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
#[derive(Clone)]
pub struct Board {
    piece_bitboard: PositionalBitboard,
    pub board: ChessBoard,

    pub castle_rights: [CastleRights; 2],
    pub side_to_move: PieceColor,
    pub en_passant: Option<Position>
}

impl Board {
    pub fn default() -> Self {
        Board {
            piece_bitboard: std::array::from_fn(|_| Bitboard::default()),
            castle_rights: std::array::from_fn(|_| CastleRights::default()),
            side_to_move: PieceColor::White,
            en_passant: None,
            board: ChessBoard::default()
        }
    }

    /// Returns all occupied squares on the board.
    pub fn occupied(&self) -> Bitboard {
        self.piece_bitboard[PieceColor::White.to_index()] | self.piece_bitboard[PieceColor::Black.to_index()]
    }

    /// Returns a bitboard for a particular color.
    pub fn color(&self, color: PieceColor) -> Bitboard {
        self.piece_bitboard[color.to_index()]
    }

    /// Returns a bitboard for a particular piece, irrespective of color.
    pub fn piece(&self, piece: PieceType) -> Bitboard {
        self.piece_bitboard[piece.to_index()]
    }

    /// Returns a bitboard for a particular piece and color.
    pub fn colored_piece(&self, piece: PieceType, color: PieceColor) -> Bitboard {
        self.piece_bitboard[piece.to_index()] & self.piece_bitboard[color.to_index()]
    }

    /// Initialises a chess board given a FEN string.
    /// 
    /// Returns an error if the FEN is invalid.
    pub fn new(fen: &str) -> Board {
        let mut chess_board = Board::default();

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
                    chess_board.board[rank as usize][file as usize] = Some(Piece::new(piece_type.clone(), piece_color));

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
        
    /// Generates all legal moves for a given piece.
    pub fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        for i in 0..8 {
            for j in 0..8 {
                if let Some(p) = self.board[i][j].clone() && p.piece_color == self.side_to_move {
                    moves.extend(p.generate_moves(self, Position::new(i as u8, j as u8)));
                }
            }
        }

        moves
    }
    
    /// Applies a move to the board.
    pub fn make_move(&mut self, piece_move: &Move, dbg: bool) {
        let initial_piece = self.board[piece_move.initial.rank as usize][piece_move.initial.file as usize].clone().expect("expected a piece on initial square");
        let end_piece = self.board[piece_move.end.rank as usize][piece_move.end.file as usize].clone();

        if dbg {
            dbg!(&initial_piece);
            dbg!(&end_piece);
        }

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

    /// Performance testing, move path enumerating function.
    pub fn perft(&self, depth: usize, initial_depth: usize, last_moves: &mut Vec<String>) -> (u64, std::time::Duration) {
        let time = std::time::Instant::now();

        if depth == 0 {
            return (1, time.elapsed());
        }

        let moves = self.generate_moves();
        let mut num_positions = 0;

        'a: for piece_move in moves.iter() {
            let mut board = self.clone();
            let mut dbg = false;

            if last_moves.len() >= 2 && last_moves[0] == "b1a3" && last_moves[1] == "a7a5" {
                dbg = true;
            }
            
            board.make_move(piece_move, dbg);

            if last_moves.len() >= 2 && last_moves[0] == "b1a3" && last_moves[1] == "a7a5" {
                println!("{} - 1", format!("{}{}", piece_move.initial.get_code(), piece_move.end.get_code()));
                board.occupied().render_bitboard(Position::new(0, 0));
            }

            for r in 0..8 {
                for f in 0..8 {
                    let pos = Position::new(r, f);

                    if board.colored_piece(PieceType::King, board.side_to_move).get_bit(pos) && pos.is_under_attack(&board, !board.side_to_move) {
                        continue 'a;
                    }
                }
            }

            let mut moves = last_moves.clone();
            moves.push(format!("{}{}", piece_move.initial.get_code(), piece_move.end.get_code()));

            board.side_to_move = !board.side_to_move;
            let new_nodes = board.perft(depth - 1, initial_depth, &mut moves).0;

            // if last_moves.len() == 1 && last_moves[0] == "b1a3" {
            //     println!("{} - {}", format!("{}{}", piece_move.initial.get_code(), piece_move.end.get_code()), new_nodes);
            // }

            // println!("{}{}{} - {}", "\t".repeat(initial_depth - depth), piece_move.initial.get_code(), piece_move.end.get_code(), new_nodes);
            // if initial_depth == depth {
            //     println!("{}{} - {}", piece_move.initial.get_code(), piece_move.end.get_code(), new_nodes);
            // }

            num_positions += new_nodes;
        }

        (num_positions, time.elapsed())
    }

    /// Generates a magic index given a magic entry and a blocker bitboard.
    pub fn generate_magic_index(magic: &MagicEntry, blockers: &Bitboard) -> usize {
        let blockers = blockers.board & magic.mask;
        let hash = blockers.wrapping_mul(magic.magic);
        let index = (hash >> magic.shift) as usize;
        magic.offset as usize + index
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f);

        for row in self.board.iter() {
            for cell in row.iter() {
                write!(f, "| ");
                match cell {
                    Some(piece) => {
                        let uppercase = piece.piece_color == PieceColor::White;
                        match piece.piece_type {
                            PieceType::Pawn => if uppercase { write!(f, "P")? } else { write!(f, "p")? },
                            PieceType::Rook => if uppercase { write!(f, "R")? } else { write!(f, "r")? },
                            PieceType::Knight => if uppercase { write!(f, "N")? } else { write!(f, "n")? },
                            PieceType::Bishop => if uppercase { write!(f, "B")? } else { write!(f, "b")? },
                            PieceType::Queen => if uppercase { write!(f, "Q")? } else { write!(f, "q")? },
                            PieceType::King => if uppercase { write!(f, "K")? } else { write!(f, "k")? },
                        }
                    },
                    None => write!(f, " ")?,
                }
            }
    
            writeln!(f, "|")?;
        }
        println!();

        std::fmt::Result::Ok(())
    }
}