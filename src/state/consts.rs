use std::collections::HashMap;
use super::piece::PieceType;

/// An array representing all the possible
/// offsets for each piece type.
pub const PIECE_OFFSETS: &[&[(i8, i8)]] = &[
    // Black Pawn
    &[
        (1, 0),  // one step forward
        (2, 0),  // two steps forward from the starting position
        (1, 1),  // capture diagonally right
        (1, -1), // capture diagonally left
    ],
    // White Pawn
    &[
        (-1, 0),  // one step backward
        (-2, 0),  // two steps backward from the starting position
        (-1, 1),  // capture diagonally right
        (-1, -1), // capture diagonally left
    ],
    // Knight
    &[
        (2, 1), (2, -1), (-2, 1), (-2, -1), // L-shape moves horizontally first
        (1, 2), (1, -2), (-1, 2), (-1, -2), // L-shape moves vertically first
    ],
    // Bishop
    &[
        (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7), // diagonals
        (1, -1), (2, -2), (3, -3), (4, -4), (5, -5), (6, -6), (7, -7), 
        (-1, 1), (-2, 2), (-3, 3), (-4, 4), (-5, 5), (-6, 6), (-7, 7),
        (-1, -1), (-2, -2), (-3, -3), (-4, -4), (-5, -5), (-6, -6), (-7, -7)
    ],
    // Rook
    &[
        (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), // vertical
        (-1, 0), (-2, 0), (-3, 0), (-4, 0), (-5, 0), (-6, 0), (-7, 0), 
        (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), // horizontal
        (0, -1), (0, -2), (0, -3), (0, -4), (0, -5), (0, -6), (0, -7)
    ],
    // Queen
    &[
        (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), // vertical
        (-1, 0), (-2, 0), (-3, 0), (-4, 0), (-5, 0), (-6, 0), (-7, 0), 
        (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6), (0, 7), // horizontal
        (0, -1), (0, -2), (0, -3), (0, -4), (0, -5), (0, -6), (0, -7),
        (1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6), (7, 7), // diagonals
        (1, -1), (2, -2), (3, -3), (4, -4), (5, -5), (6, -6), (7, -7), 
        (-1, 1), (-2, 2), (-3, 3), (-4, 4), (-5, 5), (-6, 6), (-7, 7),
        (-1, -1), (-2, -2), (-3, -3), (-4, -4), (-5, -5), (-6, -6), (-7, -7)
    ],
    // King
    &[
        (1, 0), (0, 1), (-1, 0), (0, -1),  // horizontal and vertical one step
        (1, 1), (1, -1), (-1, 1), (-1, -1) // diagonals one step
    ],
];

lazy_static::lazy_static! {
    pub static ref PIECE_MAP: HashMap<char, PieceType> = vec![
        ('p', PieceType::Pawn),
        ('n', PieceType::Knight),
        ('b', PieceType::Bishop),
        ('r', PieceType::Rook),
        ('q', PieceType::Queen),
        ('k', PieceType::King),
    ].into_iter().collect();
}