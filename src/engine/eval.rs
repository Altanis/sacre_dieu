use crate::utils::{board::Board, consts, piece::{PieceColor, PieceType}};

/// Evaluates the board, where negative values represent a black advantage and positive
/// values represent a white advantage.
pub fn evaluate_board(board: &Board) -> i32 {
    let eval = count_material(board, PieceColor::White) as i32 - count_material(board, PieceColor::Black) as i32;
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