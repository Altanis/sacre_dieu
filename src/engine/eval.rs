use strum::IntoEnumIterator;

use crate::utils::{board::{Bitboard, Board}, consts::{self, PIECE_SQUARE_TABLE}, piece::{PieceColor, PieceType}};

/// Evaluates the board, where negative values represent a black advantage and positive
/// values represent a white advantage.
pub fn evaluate_board(board: &Board) -> i32 {
    let (mut mg, mut eg) = (0_i32, 0_i32);
    
    let material_eval = count_material(board, PieceColor::White) as i32 - count_material(board, PieceColor::Black) as i32;
    mg += material_eval;
    eg += material_eval;

    let (psqt_white_mg, psqt_white_eg) = evaluate_piece_square_score(board, PieceColor::White);
    let (psqt_black_mg, psqt_black_eg) = evaluate_piece_square_score(board, PieceColor::Black);

    mg += psqt_white_mg - psqt_black_mg;
    eg += psqt_white_eg - psqt_black_eg;

    let phase = (board.phase() as i32).min(24);
    let eval = (mg * phase + eg * (24 - phase)) / 24;
    let perspective = if board.side_to_move == PieceColor::White { 1 } else { -1 };

    eval * perspective
}

/// Counts the material for a side of the board.
pub fn count_material(board: &Board, side: PieceColor) -> u32 {
    let pawn_bitboard = board.colored_piece(PieceType::Pawn, side);
    let knight_bitboard = board.colored_piece(PieceType::Knight, side);
    let bishop_bitboard = board.colored_piece(PieceType::Bishop, side);
    let rook_bitboard = board.colored_piece(PieceType::Rook, side);
    let queen_bitboard = board.colored_piece(PieceType::Queen, side);

    let pawn_material = pawn_bitboard.board.count_ones() * consts::PAWN_VALUE;
    let knight_material = knight_bitboard.board.count_ones() * consts::KNIGHT_VALUE;
    let bishop_material = bishop_bitboard.board.count_ones() * consts::BISHOP_VALUE;
    let rook_material = rook_bitboard.board.count_ones() * consts::ROOK_VALUE;
    let queen_material = queen_bitboard.board.count_ones() * consts::QUEEN_VALUE;

    pawn_material + knight_material + bishop_material + rook_material + queen_material
}

/// Evaluates a piece square score for a certain side.
pub fn evaluate_piece_square_score(board: &Board, side: PieceColor) -> (i32, i32) {
    let mut mg = 0_i32;
    let mut eg = 0_i32;

    for piece_type in PieceType::iter() {
        let piece_index = piece_type.clone() as usize;
        let mut piece_bitboard = board.colored_piece(piece_type, side);

        while piece_bitboard != Bitboard::ZERO {
            let tile = piece_bitboard.pop_lsb();
            let tile_index = if side == PieceColor::White { tile.index() ^ 56 } else { tile.index() };

            let (opening_eval, endgame_eval) = PIECE_SQUARE_TABLE[piece_index][tile_index];
            mg += opening_eval;
            eg += endgame_eval;
        }
    }

    (mg, eg)
}