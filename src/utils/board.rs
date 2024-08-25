use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use strum::EnumCount;
use crate::{engine::eval::{self, evaluate_board}, utils::consts::I32_NEGATIVE_INFINITY};

use super::{consts::{get_bishop_mask, get_rook_mask, MagicEntry, BISHOP_MAGICS, BLACK_PAWN_MASK, KING_MASKS, KNIGHT_MASKS, MAX_LEGAL_MOVES, PIECE_MAP, ROOK_MAGICS, WHITE_PAWN_MASK}, piece::*};
use colored::Colorize;

/// A type representing an array of bitboards for tracking piece/color state.
pub type PositionalBitboard = [Bitboard; PieceType::COUNT + PieceColor::COUNT];

/// A bitboard representing the presence of a specific state on the chess board.
#[derive(Debug, Default, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Bitboard {
    /// The underlying u64 board. todo make this private
    pub board: u64
}

impl Bitboard {
    /// Instantiates a bitboard.
    pub fn new(board: u64) -> Self {
        Bitboard { board }
    }

    /// Instantiates a constant ZERO bitboard.
    pub const fn const_zero() -> Self {
        Bitboard { board: 0 }
    }

    /// Sets a state on the board, given a tile.
    pub fn set_bit(&mut self, tile: Tile) {
        self.board |= 1 << tile.index();
    }

    /// Clears a state on the board, given a tile.
    pub fn clear_bit(&mut self, tile: Tile) {
        self.board &= !(1 << tile.index());
    }

    /// Checks if a state is set on the board, given a tile.
    pub fn get_bit(&self, tile: Tile) -> bool {
        self.board & (1 << tile.index()) != 0
    }

    /// Renders the bitboard.
    pub fn render_bitboard(&self, tile: Tile) {
        for row in (0..8).rev() {
            for col in 0..8 {
                let current_tile = Tile::new(row, col).unwrap();
                let is_set = self.get_bit(current_tile);
    
                print!( "| ");
                if current_tile == tile {
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

    /// Pops the LSB from the internal bitboard, returning a tile at the index of the first set bit.
    pub fn pop_lsb(&mut self) -> Tile {
        let lsb_index = self.board.trailing_zeros();
        self.board &= self.board - 1;

        let r = lsb_index as u8 / 8;
        let f = lsb_index as u8 % 8;
        
        Tile::new(r, f).unwrap()
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
    pub board: [Option<Piece>; 64],

    pub castle_rights: [CastleRights; 2],
    pub side_to_move: PieceColor,
    pub en_passant: Option<Tile>
}

impl Board {
    pub fn default() -> Self {
        Board {
            piece_bitboard: std::array::from_fn(|_| Bitboard::default()),
            castle_rights: std::array::from_fn(|_| CastleRights::default()),
            side_to_move: PieceColor::White,
            en_passant: None,
            board: std::array::from_fn(|_| None)
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

    /// Whether or not the king of a specific color is in check.
    pub fn in_check(&self, color: PieceColor) -> bool {
        let mut king_bitboard = self.colored_piece(PieceType::King, color);
        let tile = king_bitboard.pop_lsb();

        tile.is_under_attack(self, !color)
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
                    chess_board.board[(rank * 8 + file) as usize] = Some(Piece::new(piece_type.clone(), piece_color));

                    chess_board.piece_bitboard[piece_type.to_index()].set_bit(Tile::new(rank, file).expect("invalid coordinate"));
                    chess_board.piece_bitboard[piece_color.to_index()].set_bit(Tile::new(rank, file).expect("invalid coordinate"));

                    file += 1;
                }
                c => panic!("invalid board notation {}", c)
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

        if Tile::is_code_valid(en_passant) {
            chess_board.en_passant = Some(Tile::from_code(en_passant));
        }
        
        chess_board
    }
        
    /// Generates all legal moves for a given piece.
    pub fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(MAX_LEGAL_MOVES);

        for square in 0..64 {
            if let Some(p) = self.board[square].clone() && p.piece_color == self.side_to_move {
                moves.extend(p.generate_moves(self, Tile::new(square as u8 / 8, square as u8 % 8).unwrap()));
            }
        }

        moves
    }
    
    /// Applies a move to the board.
    pub fn make_move(&self, piece_move: &Move) -> Option<Board> {
        let mut board = self.clone();
        
        let initial_piece = board.board[piece_move.initial.index()].clone().expect("expected a piece on initial square");
        let end_piece = board.board[piece_move.end.index()].clone();

        // Update the bitboards.
        if let Some(ref piece) = end_piece {
            board.piece_bitboard[piece.piece_type.to_index()].clear_bit(piece_move.end);
            board.piece_bitboard[piece.piece_color.to_index()].clear_bit(piece_move.end);
        }

        board.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.initial);
        board.piece_bitboard[initial_piece.piece_color.to_index()].clear_bit(piece_move.initial);
        board.piece_bitboard[initial_piece.piece_type.to_index()].set_bit(piece_move.end);
        board.piece_bitboard[initial_piece.piece_color.to_index()].set_bit(piece_move.end);

        // Update the mailbox board.
        board.board[piece_move.initial.index()] = None;
        board.board[piece_move.end.index()] = Some(initial_piece.clone());

        let castle_rights = &mut board.castle_rights[initial_piece.piece_color.to_index()];
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
        }

        if let Some(ref piece) = end_piece && piece.piece_type == PieceType::Rook {
            // Can't castle with a dead rook
            let castle_rights = &mut board.castle_rights[piece.piece_color.to_index()];
            let correct_rank = if piece.piece_color == PieceColor::White { 0 } else { 7 };

            if *castle_rights != CastleRights::None && piece_move.end.rank == correct_rank {
                if piece_move.end.file == 0 {
                    *castle_rights = if *castle_rights == CastleRights::QueenSide { CastleRights::None } else { CastleRights::KingSide };
                } else if piece_move.end.file == 7 {
                    *castle_rights = if *castle_rights == CastleRights::KingSide { CastleRights::None } else { CastleRights::QueenSide };
                }
            }
        }

        board.en_passant = None;

        match piece_move.metadata {
            MoveFlags::DoublePush => {
                let direction = if initial_piece.piece_color == PieceColor::White { -1 } else { 1 };
                board.en_passant = Some(piece_move.end.transform(direction, 0).unwrap());
            },
            MoveFlags::EnPassant => {
                let direction = if initial_piece.piece_color == PieceColor::White { -1 } else { 1 };
                let capture_position = piece_move.end.transform(direction * 1, 0).unwrap();

                let piece = &board.board[capture_position.index()].as_ref().expect("en passant on a nothing piece");

                board.piece_bitboard[piece.piece_type.to_index()].clear_bit(capture_position);
                board.piece_bitboard[piece.piece_color.to_index()].clear_bit(capture_position);
                board.board[capture_position.index()] = None;
            },
            MoveFlags::Castling => {
                board.castle_rights[initial_piece.piece_color.to_index()] = CastleRights::None;
                let king_side = (piece_move.end.file - piece_move.initial.file) == 2;

                let old_rook_tile = Tile::new(
                    if initial_piece.piece_color == PieceColor::White { 0 } else { 7 }, 
                    if king_side { 7 } else { 0 }
                ).unwrap();

                let new_rook_tile = Tile::new(
                    if initial_piece.piece_color == PieceColor::White { 0 } else { 7 }, 
                    if king_side { 5 } else { 3 }
                ).unwrap();

                board.piece_bitboard[PieceType::Rook.to_index()].clear_bit(old_rook_tile);
                board.piece_bitboard[initial_piece.piece_color.to_index()].clear_bit(old_rook_tile);
                board.piece_bitboard[PieceType::Rook.to_index()].set_bit(new_rook_tile);
                board.piece_bitboard[initial_piece.piece_color.to_index()].set_bit(new_rook_tile);

                board.board[old_rook_tile.index()] = None;
                board.board[new_rook_tile.index()] = Some(Piece::new(PieceType::Rook, initial_piece.piece_color));
            },
            MoveFlags::KnightPromotion => {
                board.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.end);
                board.piece_bitboard[PieceType::Knight.to_index()].set_bit(piece_move.end);
                board.board[piece_move.end.index()] = Some(Piece::new(PieceType::Knight, initial_piece.piece_color));
            },
            MoveFlags::BishopPromotion => {
                board.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.end);
                board.piece_bitboard[PieceType::Bishop.to_index()].set_bit(piece_move.end);
                board.board[piece_move.end.index()] = Some(Piece::new(PieceType::Bishop, initial_piece.piece_color));
            },
            MoveFlags::RookPromotion => {
                board.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.end);
                board.piece_bitboard[PieceType::Rook.to_index()].set_bit(piece_move.end);
                board.board[piece_move.end.index()] = Some(Piece::new(PieceType::Rook, initial_piece.piece_color));
            },
            MoveFlags::QueenPromotion => {
                board.piece_bitboard[initial_piece.piece_type.to_index()].clear_bit(piece_move.end);
                board.piece_bitboard[PieceType::Queen.to_index()].set_bit(piece_move.end);
                board.board[piece_move.end.index()] = Some(Piece::new(PieceType::Queen, initial_piece.piece_color));
            },
            MoveFlags::None => {}
        }

        if board.in_check(board.side_to_move) {
            None
        } else {
            board.side_to_move = !board.side_to_move;
            Some(board)
        }
    }

    /// Searches for a move with the highest evaluation.
    pub fn search(&self, depth: usize, mut alpha: i32, beta: i32) -> (i32, u64, Option<Move>) {
        if depth == 0 {
            return (eval::evaluate_board(self), 1, None);
        }

        let moves = self.generate_moves();

        if moves.is_empty() {
            if self.in_check(self.side_to_move) {
                return (I32_NEGATIVE_INFINITY + (depth as i32), 0, None); // Checkmate.
            } else {
                return (0, 0, None); // Stalemate.
            }
        }

        let mut best_move = None;
        let mut best_eval = I32_NEGATIVE_INFINITY;
        let mut nodes = 0;

        for piece_move in moves.iter() {
            if let Some(board) = self.make_move(piece_move) {
                let (mut eval, positions, _) = board.search(depth - 1, -beta, -alpha);
                
                eval *= -1;
                nodes += positions;
    
                if eval > best_eval || best_move.is_none() {
                    best_eval = eval;
                    best_move = Some(piece_move.clone());
                }
    
                if eval >= beta {
                    return (beta, nodes, best_move);
                }
    
                if eval > alpha {
                    alpha = eval;
                }
            }
        }

        (best_eval, nodes, best_move)
    }

    /// Performance testing, move path enumerating function.
    pub fn perft(&self, depth: usize) -> u64 {
        if depth == 0 {
            return 1;
        }

        let moves = self.generate_moves();
        let mut num_moves = 0;

        for piece_move in moves.iter() {
            if let Some(board) = self.make_move(piece_move) {
                num_moves += board.perft(depth - 1);
            }
        }

        num_moves
    }

    pub fn debug_perft(&self, depth: usize, initial_depth: usize, last_moves: &mut Vec<String>) -> (u64, std::time::Duration) {
        let time = std::time::Instant::now();

        if depth == 0 {
            return (1, time.elapsed());
        }

        let moves = self.generate_moves();
        let mut num_positions = 0;

        for piece_move in moves.iter() {
            let cur_code = piece_move.to_uci();

            let mut board = self.clone();
            // let dbg = last_moves.len() == 3 && last_moves[0] == "f1f2" && last_moves[1] == "b2a1n" && last_moves[2] == "d1c2";
            // let dbg = last_moves.len() == 3 && last_moves[0] == "f1f2" && last_moves[1] == "b2a1r" && last_moves[2] == "d1a1";
            let dbg = last_moves.len() == 4 && last_moves[0] == "h1g2" && last_moves[1] == "a1b2" && last_moves[2] == "g2f1" && last_moves[3] == "b2a1";

            if let Some(board) = self.make_move(piece_move) {    
                let mut moves = last_moves.clone();
                moves.push(cur_code.clone());
    
                let new_nodes = board.debug_perft(depth - 1, initial_depth, &mut moves).0;
    
                if dbg {
                    println!("{} - {}", cur_code, new_nodes);
                }
    
                num_positions += new_nodes;
            }
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

        let board_size = 8;
        
        for row in (0..board_size).rev() {
            for col in 0..board_size {
                let index = row * board_size + col;
                write!(f, "| ")?;
                
                match &self.board[index] {
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
        
        writeln!(f);
        

        std::fmt::Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::board::Board;
    use colored::Colorize;

    const EPD_FILE: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ;D1 20 ;D2 400 ;D3 8902 ;D4 197281 ;D5 4865609 ;D6 119060324
r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ;D1 48 ;D2 2039 ;D3 97862 ;D4 4085603 ;D5 193690690
4k3/8/8/8/8/8/8/4K2R w K - 0 1 ;D1 15 ;D2 66 ;D3 1197 ;D4 7059 ;D5 133987 ;D6 764643
4k3/8/8/8/8/8/8/R3K3 w Q - 0 1 ;D1 16 ;D2 71 ;D3 1287 ;D4 7626 ;D5 145232 ;D6 846648
4k2r/8/8/8/8/8/8/4K3 w k - 0 1 ;D1 5 ;D2 75 ;D3 459 ;D4 8290 ;D5 47635 ;D6 899442
r3k3/8/8/8/8/8/8/4K3 w q - 0 1 ;D1 5 ;D2 80 ;D3 493 ;D4 8897 ;D5 52710 ;D6 1001523
4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1 ;D1 26 ;D2 112 ;D3 3189 ;D4 17945 ;D5 532933 ;D6 2788982
r3k2r/8/8/8/8/8/8/4K3 w kq - 0 1 ;D1 5 ;D2 130 ;D3 782 ;D4 22180 ;D5 118882 ;D6 3517770
8/8/8/8/8/8/6k1/4K2R w K - 0 1 ;D1 12 ;D2 38 ;D3 564 ;D4 2219 ;D5 37735 ;D6 185867
8/8/8/8/8/8/1k6/R3K3 w Q - 0 1 ;D1 15 ;D2 65 ;D3 1018 ;D4 4573 ;D5 80619 ;D6 413018
4k2r/6K1/8/8/8/8/8/8 w k - 0 1 ;D1 3 ;D2 32 ;D3 134 ;D4 2073 ;D5 10485 ;D6 179869
r3k3/1K6/8/8/8/8/8/8 w q - 0 1 ;D1 4 ;D2 49 ;D3 243 ;D4 3991 ;D5 20780 ;D6 367724
r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1 ;D1 26 ;D2 568 ;D3 13744 ;D4 314346 ;D5 7594526 ;D6 179862938
r3k2r/8/8/8/8/8/8/1R2K2R w Kkq - 0 1 ;D1 25 ;D2 567 ;D3 14095 ;D4 328965 ;D5 8153719 ;D6 195629489
r3k2r/8/8/8/8/8/8/2R1K2R w Kkq - 0 1 ;D1 25 ;D2 548 ;D3 13502 ;D4 312835 ;D5 7736373 ;D6 184411439
r3k2r/8/8/8/8/8/8/R3K1R1 w Qkq - 0 1 ;D1 25 ;D2 547 ;D3 13579 ;D4 316214 ;D5 7878456 ;D6 189224276
1r2k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1 ;D1 26 ;D2 583 ;D3 14252 ;D4 334705 ;D5 8198901 ;D6 198328929
2r1k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1 ;D1 25 ;D2 560 ;D3 13592 ;D4 317324 ;D5 7710115 ;D6 185959088
r3k1r1/8/8/8/8/8/8/R3K2R w KQq - 0 1 ;D1 25 ;D2 560 ;D3 13607 ;D4 320792 ;D5 7848606 ;D6 190755813
4k3/8/8/8/8/8/8/4K2R b K - 0 1 ;D1 5 ;D2 75 ;D3 459 ;D4 8290 ;D5 47635 ;D6 899442
4k3/8/8/8/8/8/8/R3K3 b Q - 0 1 ;D1 5 ;D2 80 ;D3 493 ;D4 8897 ;D5 52710 ;D6 1001523
4k2r/8/8/8/8/8/8/4K3 b k - 0 1 ;D1 15 ;D2 66 ;D3 1197 ;D4 7059 ;D5 133987 ;D6 764643
r3k3/8/8/8/8/8/8/4K3 b q - 0 1 ;D1 16 ;D2 71 ;D3 1287 ;D4 7626 ;D5 145232 ;D6 846648
4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1 ;D1 5 ;D2 130 ;D3 782 ;D4 22180 ;D5 118882 ;D6 3517770
r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1 ;D1 26 ;D2 112 ;D3 3189 ;D4 17945 ;D5 532933 ;D6 2788982
8/8/8/8/8/8/6k1/4K2R b K - 0 1 ;D1 3 ;D2 32 ;D3 134 ;D4 2073 ;D5 10485 ;D6 179869
8/8/8/8/8/8/1k6/R3K3 b Q - 0 1 ;D1 4 ;D2 49 ;D3 243 ;D4 3991 ;D5 20780 ;D6 367724
4k2r/6K1/8/8/8/8/8/8 b k - 0 1 ;D1 12 ;D2 38 ;D3 564 ;D4 2219 ;D5 37735 ;D6 185867
r3k3/1K6/8/8/8/8/8/8 b q - 0 1 ;D1 15 ;D2 65 ;D3 1018 ;D4 4573 ;D5 80619 ;D6 413018
r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1 ;D1 26 ;D2 568 ;D3 13744 ;D4 314346 ;D5 7594526 ;D6 179862938
r3k2r/8/8/8/8/8/8/1R2K2R b Kkq - 0 1 ;D1 26 ;D2 583 ;D3 14252 ;D4 334705 ;D5 8198901 ;D6 198328929
r3k2r/8/8/8/8/8/8/2R1K2R b Kkq - 0 1 ;D1 25 ;D2 560 ;D3 13592 ;D4 317324 ;D5 7710115 ;D6 185959088
r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1 ;D1 25 ;D2 560 ;D3 13607 ;D4 320792 ;D5 7848606 ;D6 190755813
1r2k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1 ;D1 25 ;D2 567 ;D3 14095 ;D4 328965 ;D5 8153719 ;D6 195629489
2r1k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1 ;D1 25 ;D2 548 ;D3 13502 ;D4 312835 ;D5 7736373 ;D6 184411439
r3k1r1/8/8/8/8/8/8/R3K2R b KQq - 0 1 ;D1 25 ;D2 547 ;D3 13579 ;D4 316214 ;D5 7878456 ;D6 189224276
8/1n4N1/2k5/8/8/5K2/1N4n1/8 w - - 0 1 ;D1 14 ;D2 195 ;D3 2760 ;D4 38675 ;D5 570726 ;D6 8107539
8/1k6/8/5N2/8/4n3/8/2K5 w - - 0 1 ;D1 11 ;D2 156 ;D3 1636 ;D4 20534 ;D5 223507 ;D6 2594412
8/8/4k3/3Nn3/3nN3/4K3/8/8 w - - 0 1 ;D1 19 ;D2 289 ;D3 4442 ;D4 73584 ;D5 1198299 ;D6 19870403
K7/8/2n5/1n6/8/8/8/k6N w - - 0 1 ;D1 3 ;D2 51 ;D3 345 ;D4 5301 ;D5 38348 ;D6 588695
k7/8/2N5/1N6/8/8/8/K6n w - - 0 1 ;D1 17 ;D2 54 ;D3 835 ;D4 5910 ;D5 92250 ;D6 688780
8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1 ;D1 15 ;D2 193 ;D3 2816 ;D4 40039 ;D5 582642 ;D6 8503277
8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1 ;D1 16 ;D2 180 ;D3 2290 ;D4 24640 ;D5 288141 ;D6 3147566
8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1 ;D1 4 ;D2 68 ;D3 1118 ;D4 16199 ;D5 281190 ;D6 4405103
K7/8/2n5/1n6/8/8/8/k6N b - - 0 1 ;D1 17 ;D2 54 ;D3 835 ;D4 5910 ;D5 92250 ;D6 688780
k7/8/2N5/1N6/8/8/8/K6n b - - 0 1 ;D1 3 ;D2 51 ;D3 345 ;D4 5301 ;D5 38348 ;D6 588695
B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1 ;D1 17 ;D2 278 ;D3 4607 ;D4 76778 ;D5 1320507 ;D6 22823890
8/8/1B6/7b/7k/8/2B1b3/7K w - - 0 1 ;D1 21 ;D2 316 ;D3 5744 ;D4 93338 ;D5 1713368 ;D6 28861171
k7/B7/1B6/1B6/8/8/8/K6b w - - 0 1 ;D1 21 ;D2 144 ;D3 3242 ;D4 32955 ;D5 787524 ;D6 7881673
K7/b7/1b6/1b6/8/8/8/k6B w - - 0 1 ;D1 7 ;D2 143 ;D3 1416 ;D4 31787 ;D5 310862 ;D6 7382896
B6b/8/8/8/2K5/5k2/8/b6B b - - 0 1 ;D1 6 ;D2 106 ;D3 1829 ;D4 31151 ;D5 530585 ;D6 9250746
8/8/1B6/7b/7k/8/2B1b3/7K b - - 0 1 ;D1 17 ;D2 309 ;D3 5133 ;D4 93603 ;D5 1591064 ;D6 29027891
k7/B7/1B6/1B6/8/8/8/K6b b - - 0 1 ;D1 7 ;D2 143 ;D3 1416 ;D4 31787 ;D5 310862 ;D6 7382896
K7/b7/1b6/1b6/8/8/8/k6B b - - 0 1 ;D1 21 ;D2 144 ;D3 3242 ;D4 32955 ;D5 787524 ;D6 7881673
7k/RR6/8/8/8/8/rr6/7K w - - 0 1 ;D1 19 ;D2 275 ;D3 5300 ;D4 104342 ;D5 2161211 ;D6 44956585
R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1 ;D1 36 ;D2 1027 ;D3 29215 ;D4 771461 ;D5 20506480 ;D6 525169084
7k/RR6/8/8/8/8/rr6/7K b - - 0 1 ;D1 19 ;D2 275 ;D3 5300 ;D4 104342 ;D5 2161211 ;D6 44956585
R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1 ;D1 36 ;D2 1027 ;D3 29227 ;D4 771368 ;D5 20521342 ;D6 524966748
6kq/8/8/8/8/8/8/7K w - - 0 1 ;D1 2 ;D2 36 ;D3 143 ;D4 3637 ;D5 14893 ;D6 391507
6KQ/8/8/8/8/8/8/7k b - - 0 1 ;D1 2 ;D2 36 ;D3 143 ;D4 3637 ;D5 14893 ;D6 391507
K7/8/8/3Q4/4q3/8/8/7k w - - 0 1 ;D1 6 ;D2 35 ;D3 495 ;D4 8349 ;D5 166741 ;D6 3370175
6qk/8/8/8/8/8/8/7K b - - 0 1 ;D1 22 ;D2 43 ;D3 1015 ;D4 4167 ;D5 105749 ;D6 419369
6KQ/8/8/8/8/8/8/7k b - - 0 1 ;D1 2 ;D2 36 ;D3 143 ;D4 3637 ;D5 14893 ;D6 391507
K7/8/8/3Q4/4q3/8/8/7k b - - 0 1 ;D1 6 ;D2 35 ;D3 495 ;D4 8349 ;D5 166741 ;D6 3370175
8/8/8/8/8/K7/P7/k7 w - - 0 1 ;D1 3 ;D2 7 ;D3 43 ;D4 199 ;D5 1347 ;D6 6249
8/8/8/8/8/7K/7P/7k w - - 0 1 ;D1 3 ;D2 7 ;D3 43 ;D4 199 ;D5 1347 ;D6 6249
K7/p7/k7/8/8/8/8/8 w - - 0 1 ;D1 1 ;D2 3 ;D3 12 ;D4 80 ;D5 342 ;D6 2343
7K/7p/7k/8/8/8/8/8 w - - 0 1 ;D1 1 ;D2 3 ;D3 12 ;D4 80 ;D5 342 ;D6 2343
8/2k1p3/3pP3/3P2K1/8/8/8/8 w - - 0 1 ;D1 7 ;D2 35 ;D3 210 ;D4 1091 ;D5 7028 ;D6 34834
8/8/8/8/8/K7/P7/k7 b - - 0 1 ;D1 1 ;D2 3 ;D3 12 ;D4 80 ;D5 342 ;D6 2343
8/8/8/8/8/7K/7P/7k b - - 0 1 ;D1 1 ;D2 3 ;D3 12 ;D4 80 ;D5 342 ;D6 2343
K7/p7/k7/8/8/8/8/8 b - - 0 1 ;D1 3 ;D2 7 ;D3 43 ;D4 199 ;D5 1347 ;D6 6249
7K/7p/7k/8/8/8/8/8 b - - 0 1 ;D1 3 ;D2 7 ;D3 43 ;D4 199 ;D5 1347 ;D6 6249
8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1 ;D1 5 ;D2 35 ;D3 182 ;D4 1091 ;D5 5408 ;D6 34822
8/8/8/8/8/4k3/4P3/4K3 w - - 0 1 ;D1 2 ;D2 8 ;D3 44 ;D4 282 ;D5 1814 ;D6 11848
4k3/4p3/4K3/8/8/8/8/8 b - - 0 1 ;D1 2 ;D2 8 ;D3 44 ;D4 282 ;D5 1814 ;D6 11848
8/8/7k/7p/7P/7K/8/8 w - - 0 1 ;D1 3 ;D2 9 ;D3 57 ;D4 360 ;D5 1969 ;D6 10724
8/8/k7/p7/P7/K7/8/8 w - - 0 1 ;D1 3 ;D2 9 ;D3 57 ;D4 360 ;D5 1969 ;D6 10724
8/8/3k4/3p4/3P4/3K4/8/8 w - - 0 1 ;D1 5 ;D2 25 ;D3 180 ;D4 1294 ;D5 8296 ;D6 53138
8/3k4/3p4/8/3P4/3K4/8/8 w - - 0 1 ;D1 8 ;D2 61 ;D3 483 ;D4 3213 ;D5 23599 ;D6 157093
8/8/3k4/3p4/8/3P4/3K4/8 w - - 0 1 ;D1 8 ;D2 61 ;D3 411 ;D4 3213 ;D5 21637 ;D6 158065
k7/8/3p4/8/3P4/8/8/7K w - - 0 1 ;D1 4 ;D2 15 ;D3 90 ;D4 534 ;D5 3450 ;D6 20960
8/8/7k/7p/7P/7K/8/8 b - - 0 1 ;D1 3 ;D2 9 ;D3 57 ;D4 360 ;D5 1969 ;D6 10724
8/8/k7/p7/P7/K7/8/8 b - - 0 1 ;D1 3 ;D2 9 ;D3 57 ;D4 360 ;D5 1969 ;D6 10724
8/8/3k4/3p4/3P4/3K4/8/8 b - - 0 1 ;D1 5 ;D2 25 ;D3 180 ;D4 1294 ;D5 8296 ;D6 53138
8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1 ;D1 8 ;D2 61 ;D3 411 ;D4 3213 ;D5 21637 ;D6 158065
8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1 ;D1 8 ;D2 61 ;D3 483 ;D4 3213 ;D5 23599 ;D6 157093
k7/8/3p4/8/3P4/8/8/7K b - - 0 1 ;D1 4 ;D2 15 ;D3 89 ;D4 537 ;D5 3309 ;D6 21104
7k/3p4/8/8/3P4/8/8/K7 w - - 0 1 ;D1 4 ;D2 19 ;D3 117 ;D4 720 ;D5 4661 ;D6 32191
7k/8/8/3p4/8/8/3P4/K7 w - - 0 1 ;D1 5 ;D2 19 ;D3 116 ;D4 716 ;D5 4786 ;D6 30980
k7/8/8/7p/6P1/8/8/K7 w - - 0 1 ;D1 5 ;D2 22 ;D3 139 ;D4 877 ;D5 6112 ;D6 41874
k7/8/7p/8/8/6P1/8/K7 w - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4354 ;D6 29679
k7/8/8/6p1/7P/8/8/K7 w - - 0 1 ;D1 5 ;D2 22 ;D3 139 ;D4 877 ;D5 6112 ;D6 41874
k7/8/6p1/8/8/7P/8/K7 w - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4354 ;D6 29679
k7/8/8/3p4/4p3/8/8/7K w - - 0 1 ;D1 3 ;D2 15 ;D3 84 ;D4 573 ;D5 3013 ;D6 22886
k7/8/3p4/8/8/4P3/8/7K w - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4271 ;D6 28662
7k/3p4/8/8/3P4/8/8/K7 b - - 0 1 ;D1 5 ;D2 19 ;D3 117 ;D4 720 ;D5 5014 ;D6 32167
7k/8/8/3p4/8/8/3P4/K7 b - - 0 1 ;D1 4 ;D2 19 ;D3 117 ;D4 712 ;D5 4658 ;D6 30749
k7/8/8/7p/6P1/8/8/K7 b - - 0 1 ;D1 5 ;D2 22 ;D3 139 ;D4 877 ;D5 6112 ;D6 41874
k7/8/7p/8/8/6P1/8/K7 b - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4354 ;D6 29679
k7/8/8/6p1/7P/8/8/K7 b - - 0 1 ;D1 5 ;D2 22 ;D3 139 ;D4 877 ;D5 6112 ;D6 41874
k7/8/6p1/8/8/7P/8/K7 b - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4354 ;D6 29679
k7/8/8/3p4/4p3/8/8/7K b - - 0 1 ;D1 5 ;D2 15 ;D3 102 ;D4 569 ;D5 4337 ;D6 22579
k7/8/3p4/8/8/4P3/8/7K b - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4271 ;D6 28662
7k/8/8/p7/1P6/8/8/7K w - - 0 1 ;D1 5 ;D2 22 ;D3 139 ;D4 877 ;D5 6112 ;D6 41874
7k/8/p7/8/8/1P6/8/7K w - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4354 ;D6 29679
7k/8/8/1p6/P7/8/8/7K w - - 0 1 ;D1 5 ;D2 22 ;D3 139 ;D4 877 ;D5 6112 ;D6 41874
7k/8/1p6/8/8/P7/8/7K w - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4354 ;D6 29679
k7/7p/8/8/8/8/6P1/K7 w - - 0 1 ;D1 5 ;D2 25 ;D3 161 ;D4 1035 ;D5 7574 ;D6 55338
k7/6p1/8/8/8/8/7P/K7 w - - 0 1 ;D1 5 ;D2 25 ;D3 161 ;D4 1035 ;D5 7574 ;D6 55338
3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1 ;D1 7 ;D2 49 ;D3 378 ;D4 2902 ;D5 24122 ;D6 199002
7k/8/8/p7/1P6/8/8/7K b - - 0 1 ;D1 5 ;D2 22 ;D3 139 ;D4 877 ;D5 6112 ;D6 41874
7k/8/p7/8/8/1P6/8/7K b - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4354 ;D6 29679
7k/8/8/1p6/P7/8/8/7K b - - 0 1 ;D1 5 ;D2 22 ;D3 139 ;D4 877 ;D5 6112 ;D6 41874
7k/8/1p6/8/8/P7/8/7K b - - 0 1 ;D1 4 ;D2 16 ;D3 101 ;D4 637 ;D5 4354 ;D6 29679
k7/7p/8/8/8/8/6P1/K7 b - - 0 1 ;D1 5 ;D2 25 ;D3 161 ;D4 1035 ;D5 7574 ;D6 55338
k7/6p1/8/8/8/8/7P/K7 b - - 0 1 ;D1 5 ;D2 25 ;D3 161 ;D4 1035 ;D5 7574 ;D6 55338
3k4/3pp3/8/8/8/8/3PP3/3K4 b - - 0 1 ;D1 7 ;D2 49 ;D3 378 ;D4 2902 ;D5 24122 ;D6 199002
8/Pk6/8/8/8/8/6Kp/8 w - - 0 1 ;D1 11 ;D2 97 ;D3 887 ;D4 8048 ;D5 90606 ;D6 1030499
n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1 ;D1 24 ;D2 421 ;D3 7421 ;D4 124608 ;D5 2193768 ;D6 37665329
8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1 ;D1 18 ;D2 270 ;D3 4699 ;D4 79355 ;D5 1533145 ;D6 28859283
n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1 ;D1 24 ;D2 496 ;D3 9483 ;D4 182838 ;D5 3605103 ;D6 71179139
8/Pk6/8/8/8/8/6Kp/8 b - - 0 1 ;D1 11 ;D2 97 ;D3 887 ;D4 8048 ;D5 90606 ;D6 1030499
n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1 ;D1 24 ;D2 421 ;D3 7421 ;D4 124608 ;D5 2193768 ;D6 37665329
8/PPPk4/8/8/8/8/4Kppp/8 b - - 0 1 ;D1 18 ;D2 270 ;D3 4699 ;D4 79355 ;D5 1533145 ;D6 28859283
n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1 ;D1 24 ;D2 496 ;D3 9483 ;D4 182838 ;D5 3605103 ;D6 71179139
8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ;D4 43238 ;D5 674624 ;D6 11030083
rnbqkb1r/ppppp1pp/7n/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3 ;D5 11139762";

    #[test]
    fn test() {
        let mut lines = EPD_FILE.split('\n');
        let length = lines.clone().count();
    
        let mut idx = 0;
        for line in lines {
            idx += 1;
    
            let mut lines = line.split(" ;");
        
            let fen = lines.next().expect("expected a FEN value.");
            let board = Board::new(fen);
    
            print!("{}", "[INIT] ".green());
            println!("Board with FEN {} initialized. ({} / {})", fen, idx, length);
    
            for depth in lines {
                let mut depth_str = depth.split(' ');
                
                let depth = depth_str.next().expect("Expected Dx")
                    .replace("D", "")
                    .parse::<usize>().expect("Depth invalid");
    
                let expected_nodes = depth_str.next().expect("Expected expected nodes")
                    .parse::<u64>().expect("Nodes invalid");
    
                let time = std::time::Instant::now();
                let nodes = board.perft(depth);
                let time = time.elapsed();
    
                if nodes == expected_nodes {
                    println!("{}", format!("[PASS] Time: {:?}, Depth: {}, Nodes: {}, NPS: {}", time, depth, nodes, (nodes as f64) / time.as_secs_f64()).green());
                } else {
                    panic!("{}", format!("[FAIL] Time: {:?}, Depth: {}, Nodes: {}, Expected Nodes: {}", time, depth, nodes, expected_nodes).red());
                }
            }
    
            println!("{}", "Board passed. Moving onto next...\n".green());
        }
    
        println!("{}", "All cases cleared!".green());
    }
}