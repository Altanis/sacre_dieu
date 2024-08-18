use strum::{EnumCount, IntoEnumIterator};
use super::{consts::{PIECE_MAP, PIECE_OFFSETS}, piece::*};
use colored::Colorize;

/// A bitboard representing the presence of a specific state on the chess board.
#[derive(Debug, Clone)]
pub struct Bitboard {
    /// The underlying u64 board.
    board: u64
}

impl Bitboard {
    /// Instantiates a bitboard.
    fn new(board: u64) -> Self {
        Bitboard { board }
    }

    /// Sets a state on the board, given a position.
    pub fn set_bit(&mut self, position: Position) {
        self.board |= 1 << (position.rank * 8 + position.file);
    }

    /// Clears a state on the board, given a position.
    pub fn clear_bit(&mut self, position: Position) {
        self.board &= !(1 << (position.rank * 8 + position.file));
    }

    /// Checks if a state is set on the board, given a position.
    pub fn get_bit(&self, position: Position) -> bool {
        self.board & (1 << (position.rank * 8 + position.file)) != 0
    }

    /// Renders the bitboard.
    pub fn render_bitboard(&self, position: Position) {
        for row in 0..8 {
            for col in 0..8 {
                let pos = Position::new(row, col).unwrap();
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

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard {
            board: 0
        }
    }
}

/// A structure representing the state of an entire chess board.
#[derive(Debug)]
pub struct Board {
    pub positional_bitboards: [Bitboard; PieceType::COUNT + PieceColor::COUNT],
    pub attack_bitboard: [[Bitboard; 64]; PieceType::COUNT + 1],
    pub rook_blockers: [Vec<Bitboard>; 64],
    pub sliding_rook_bitboard: [Vec<Bitboard>; 64],
    pub bishop_blockers: [Vec<Bitboard>; 64],
    pub sliding_bishop_bitboard: [Vec<Bitboard>; 64],

    pub castle_rights: [CastleRights; 2],
    pub side_to_move: PieceColor,
    pub en_passant: Option<Position>,
    
    pub board: ChessBoard
}

impl Board {
    pub fn default() -> Self {
        Board {
            positional_bitboards: std::array::from_fn(|_| Bitboard::default()),
            attack_bitboard: std::array::from_fn(|_| std::array::from_fn(|_| Bitboard::default())),
            rook_blockers: std::array::from_fn(|_| Vec::new()),
            sliding_rook_bitboard: std::array::from_fn(|_| Vec::new()),
            bishop_blockers: std::array::from_fn(|_| Vec::new()),
            sliding_bishop_bitboard: std::array::from_fn(|_| Vec::new()),
            castle_rights: std::array::from_fn(|_| CastleRights::default()),
            side_to_move: PieceColor::White,
            en_passant: None,
            board: ChessBoard::default()
        }
    }

    /// Initialises a chess board given a FEN string.
    /// 
    /// Returns an error if the FEN is invalid.
    pub fn new(fen: &str) -> Result<Board, ()> {
        let mut chess_board = Board::default();
        chess_board.compute_attack_bitboards();
        chess_board.compute_magic_bitboards();

        let (mut rank, mut file) = (7_u8, 0_u8);

        let tokens: Vec<&str> = fen.split(' ').collect();
        if tokens.len() < 4 {
            return Err(());
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
                    let piece_type = PIECE_MAP.get(&char.to_ascii_lowercase()).ok_or(())?.clone();
                    chess_board.board[rank as usize][file as usize] = Some(Piece::new(piece_type.clone(), piece_color.clone()));

                    chess_board.positional_bitboards[piece_type.to_index()].set_bit(Position::new(rank, file)?);
                    chess_board.positional_bitboards[piece_color.to_index()].set_bit(Position::new(rank, file)?);

                    file += 1;
                }
                _ => return Err(())
            }
        }

        match side.to_ascii_lowercase().as_str() {
            "w" => chess_board.side_to_move = PieceColor::White,
            "b" => chess_board.side_to_move = PieceColor::Black,
            _ => return Err(())
        };

        for (king_side, queen_side, color) in [("K", "Q", PieceColor::White), ("k", "q", PieceColor::Black)].iter() {
            chess_board.castle_rights[color.to_index()] = match (castle_rights.contains(king_side), castle_rights.contains(queen_side)) {
                (true, true) => CastleRights::Both,
                (true, false) => CastleRights::KingSide,
                (false, true) => CastleRights::QueenSide,
                _ => CastleRights::None,
            };
        }

        if let Ok(position) = Position::from_code(en_passant) {
            chess_board.en_passant = Some(position);
        }
        
        Ok(chess_board)
    }

    /// Generates the FEN for the current board.
    pub fn generate_fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                if let Some(piece) = &self.board[rank][file] {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }

                    let piece_char = PIECE_MAP.iter().find(|(_, v)| **v == piece.piece_type).unwrap().0;
                    if piece.piece_color == PieceColor::White {
                        fen.push(piece_char.to_ascii_uppercase());
                    } else {
                        fen.push(*piece_char);
                    }
                } else {
                    empty += 1;
                }
            }

            if empty > 0 {
                fen.push_str(&empty.to_string());
            }

            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');

        fen.push_str(match self.side_to_move {
            PieceColor::White => "w",
            PieceColor::Black => "b"
        });

        fen.push(' ');

        let mut castle_rights = String::new();
        for (king_side, queen_side, color) in [("K", "Q", PieceColor::White), ("k", "q", PieceColor::Black)].iter() {
            if self.castle_rights[color.to_index()] == CastleRights::Both {
                castle_rights.push_str(king_side);
                castle_rights.push_str(queen_side);
            } else if self.castle_rights[color.to_index()] == CastleRights::KingSide {
                castle_rights.push_str(king_side);
            } else if self.castle_rights[color.to_index()] == CastleRights::QueenSide {
                castle_rights.push_str(queen_side);
            }
        }

        if castle_rights.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&castle_rights);
        }

        fen.push(' ');

        if let Some(en_passant) = self.en_passant {
            fen.push_str(&en_passant.to_code());
        } else {
            fen.push('-');
        }

        fen
    }

    /// Generates all possible moves for a given piece.
    pub fn generate_moves(&mut self, depth: u8) -> Vec<Move> {
        let mut moves = Vec::new();

        // use bitbards to generate moves


        moves
    }
    
    /// Precomputes a list of attack bitboards.
    fn compute_attack_bitboards(&mut self) {
        for square in 0..64 {
            let rank = (square / 8) as i8;
            let file = (square % 8) as i8;

            for index in 0..self.attack_bitboard.len() {
                let mut attacks: u64 = 0;

                for (dr, df) in PIECE_OFFSETS[index].iter() {
                    let (new_rank, new_file) = (rank + dr, file + df);
                    let bitboard = if !(0..8).contains(&new_rank) || !(0..8).contains(&new_file) {
                        0
                    } else {
                        1u64 << (new_rank * 8 + new_file)
                    };

                    attacks |= bitboard;
                }

                self.attack_bitboard[index][square] = Bitboard::new(attacks);
            }
        }
    }

    /// Precomputes the entire list of blocker bitboards.
    fn compute_magic_bitboards(&mut self) {
        for rank in 0..8 {
            for file in 0..8 {
                let rook_blocker_bitboard = self.compute_blocker_bitboards(rank, file, PieceType::Rook);
                let bishop_blocker_bitboard = self.compute_blocker_bitboards(rank, file, PieceType::Bishop);

                self.rook_blockers[rank * 8 + file] = rook_blocker_bitboard;
                self.bishop_blockers[rank * 8 + file] = bishop_blocker_bitboard;

                for blocker_bitboard in self.rook_blockers[rank * 8 + file].iter() {
                    let mut legal_moves = Bitboard::new(0);

                    for rank2 in rank..7 {
                        let current_position = Position::new(rank2 as u8, file as u8).unwrap();
                        if current_position == Position::new(rank as u8, file as u8).unwrap() { continue; }

                        legal_moves.set_bit(current_position);
                        if blocker_bitboard.get_bit(current_position) {
                            break;
                        }
                    }

                    for rank2 in (1..rank).rev() {
                        let current_position = Position::new(rank2 as u8, file as u8).unwrap();
                        if current_position == Position::new(rank as u8, file as u8).unwrap() { continue; }

                        legal_moves.set_bit(current_position);
                        if blocker_bitboard.get_bit(current_position) {
                            break;
                        }
                    }

                    for file2 in file..7 {
                        let current_position = Position::new(rank as u8, file2 as u8).unwrap();
                        if current_position == Position::new(rank as u8, file as u8).unwrap() { continue; }

                        legal_moves.set_bit(current_position);
                        if blocker_bitboard.get_bit(current_position) {
                            break;
                        }
                    }

                    for file2 in (1..file).rev() {
                        let current_position = Position::new(rank as u8, file2 as u8).unwrap();
                        if current_position == Position::new(rank as u8, file as u8).unwrap() { continue; }

                        legal_moves.set_bit(current_position);
                        if blocker_bitboard.get_bit(current_position) {
                            break;
                        }
                    }

                    self.sliding_rook_bitboard[rank * 8 + file].push(legal_moves);
                }

                for blocker_bitboard in self.bishop_blockers[rank * 8 + file].iter() {       
                    let mut legal_moves = Bitboard::new(0);

                    for offset in 1..7 {
                        let current_position = Position::new((rank as i8 + offset) as u8, (file as i8 + offset) as u8);
                        let next_position = Position::new((rank as i8 + offset + 1) as u8, (file as i8 + offset + 1) as u8);
                        if let Ok(pos) = current_position && next_position.is_ok() {
                            if pos == Position::new(rank as u8, file as u8).unwrap() { continue; }
                            legal_moves.set_bit(pos);
                            if blocker_bitboard.get_bit(pos) { break; }
                        } else {
                            break;
                        }
                    }
                    
                    for offset in 1..7 {
                        let current_position = Position::new((rank as i8 + offset) as u8, (file as i8 - offset) as u8);
                        let next_position = Position::new((rank as i8 + offset + 1) as u8, (file as i8 - offset + 1) as u8);
                        if let Ok(pos) = current_position && next_position.is_ok() {
                            if pos == Position::new(rank as u8, file as u8).unwrap() { continue; }
                            legal_moves.set_bit(pos);
                            if blocker_bitboard.get_bit(pos) { break; }
                        } else {
                            break;
                        }
                    }
                    
                    for offset in 1..7 {
                        let current_position = Position::new((rank as i8 - offset) as u8, (file as i8 + offset) as u8);
                        let next_position = Position::new((rank as i8 - offset + 1) as u8, (file as i8 + offset + 1) as u8);
                        if let Ok(pos) = current_position && next_position.is_ok() {
                            if pos == Position::new(rank as u8, file as u8).unwrap() { continue; }
                            legal_moves.set_bit(pos);
                            if blocker_bitboard.get_bit(pos) { break; }
                        } else {
                            break;
                        }
                    }
                    
                    for offset in 1..7 {
                        let current_position = Position::new((rank as i8 - offset) as u8, (file as i8 - offset) as u8);
                        let next_position = Position::new((rank as i8 - offset + 1) as u8, (file as i8 - offset + 1) as u8);
                        if let Ok(pos) = current_position && next_position.is_ok() {
                            if pos == Position::new(rank as u8, file as u8).unwrap() { continue; }
                            legal_moves.set_bit(pos);
                            if blocker_bitboard.get_bit(pos) { break; }
                        } else {
                            break;
                        }
                    }
                    
                    self.sliding_bishop_bitboard[rank * 8 + file].push(legal_moves);
                }
            }
        }
    }

    /// Precomputes the blocker bitboards for a slidng piece.
    pub fn compute_blocker_bitboards(&mut self, rank: usize, file: usize, piece_type: PieceType) -> Vec<Bitboard> {
        let attack_bitboard = self.attack_bitboard[piece_type.to_index() - 1][rank * 8 + file].clone();

        let mut move_square_indices = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                if attack_bitboard.get_bit(Position::new(rank, file).unwrap()) {
                    move_square_indices.push(rank * 8 + file);
                }
            }
        }

        let num_combos = 1 << move_square_indices.len();
        let mut blocker_bitboards: Vec<Bitboard> = Vec::with_capacity(num_combos);

        for pattern_idx in 0..num_combos {
            blocker_bitboards.push(Bitboard::default());

            for bit_idx in 0..move_square_indices.len() {
                let bit: u64 = ((pattern_idx >> bit_idx) & 1) as u64;
                blocker_bitboards[pattern_idx].board |= bit << move_square_indices[bit_idx];
            }
        }

        blocker_bitboards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new_valid() {
        let pos = Position::new(0, 0);
        assert!(pos.is_ok());
    }

    #[test]
    fn test_position_new_invalid() {
        let pos = Position::new(8, 0);
        assert!(pos.is_err());
    }

    #[test]
    fn test_position_from_code_valid() {
        let pos = Position::from_code("a1");
        assert_eq!(pos.unwrap(), Position::new(0, 0).unwrap());
    }

    #[test]
    fn test_position_from_code_invalid() {
        let pos = Position::from_code("z9");
        assert!(pos.is_err());
    }

    #[test]
    fn test_bitboard_set_clear_state() {
        let mut board = Bitboard::default();
        let pos = Position::new(0, 0).unwrap();
        board.set_bit(pos.clone());
        assert!(board.get_bit(pos.clone()));
        board.clear_bit(pos.clone());
        assert!(!board.get_bit(pos));
    }

    #[test]
    fn test_board_from_fen_valid() {
        let board = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert!(board.is_ok());
        let board = board.unwrap();
        assert_eq!(board.side_to_move, PieceColor::White);
        assert!(board.castle_rights[0] == CastleRights::Both); // White has both castling rights
        assert!(board.castle_rights[1] == CastleRights::Both); // Black has both castling rights

        for i in 0..7 {
            assert!(board.positional_bitboards[PieceType::Pawn.to_index()].get_bit(Position::new(1, i).unwrap()));
            assert!(board.positional_bitboards[PieceType::Pawn.to_index()].get_bit(Position::new(6, i).unwrap()));
        }
    }

    #[test]
    fn test_board_from_fen_invalid() {
        let board = Board::new("invalid fen");
        assert!(board.is_err());
    }

    #[test]
    fn test_castle_rights_parsing() {
        let board = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Kq - 0 1").unwrap();
        assert_eq!(board.castle_rights[PieceColor::White.to_index()], CastleRights::KingSide);
        assert_eq!(board.castle_rights[PieceColor::Black.to_index()], CastleRights::QueenSide);
    }

    #[test]
    fn test_en_passant_parsing() {
        let board = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq e3 0 1").unwrap();
        assert!(board.en_passant.is_some());
        let en_passant_pos = board.en_passant.unwrap();
        assert_eq!(en_passant_pos, Position::new(3, 4).unwrap());
    }
}