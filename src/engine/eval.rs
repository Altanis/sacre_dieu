use strum::IntoEnumIterator;

use crate::utils::{board::{Bitboard, Board}, consts::{self, PIECE_SQUARE_TABLE}, piece::{PieceColor, PieceType}};

/// Evaluates the board, where negative values represent a black advantage and positive
/// values represent a white advantage.
pub fn evaluate_board(board: &Board) -> i32 {
    let mut eval = 0;
    let perspective = if board.side_to_move == PieceColor::White { 1 } else { -1 };
    
    let material_eval = count_material(board, PieceColor::White) as i32 - count_material(board, PieceColor::Black) as i32;
    eval += material_eval * perspective;

    let (white_endgame_weight, black_endgame_weight) = (board.endgame_weight(PieceColor::White), board.endgame_weight(PieceColor::Black));
    let piece_square_eval = evaluate_piece_square_score(board, PieceColor::White, black_endgame_weight) as i32 - evaluate_piece_square_score(board, PieceColor::Black, white_endgame_weight) as i32;
    eval += piece_square_eval * perspective;

    eval
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
pub fn evaluate_piece_square_score(board: &Board, side: PieceColor, endgame_weight: u32) -> u32 {
    // todo "track a total midgame and total endgame score and interpolate at the end"

    let mut value = 0;

    for piece_type in PieceType::iter() {
        let index = (if side == PieceColor::Black { 6 } else { 0 }) + piece_type.clone() as usize;
        let mut piece_bitboard = board.colored_piece(piece_type, side);

        while piece_bitboard != Bitboard::ZERO() {
            let tile = piece_bitboard.pop_lsb();

            let (opening_eval, endgame_eval) = PIECE_SQUARE_TABLE[index][tile.index()];
            let nuance = opening_eval as f32 + (endgame_eval as f32 - opening_eval as f32) * (endgame_weight as f32 / 100.0);
            value += nuance as u32;
        }
    }

    value
}