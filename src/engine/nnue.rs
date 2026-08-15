//! Prophet NNUE evaluation, borrowed from the Orca engine.
//!
//! The net is a mono-accumulator 768 -> 256 -> 1 network, i16 quantized with a
//! scaling factor of 64, producing centipawns from white's perspective before
//! the side-to-move negation. Feature indices do not depend on the king square,
//! so there are no refresh buckets: a position is evaluated by activating every
//! occupied square and running the accumulator.
//!
//! The weights in `net.bin` are exported by `prophet/src/bin/export_net.rs`,
//! which quantizes the original `nnue.npz`. Evaluation is reimplemented here
//! rather than linked from the prophet crate because prophet depends on dfdx,
//! which does not build on the nightly toolchain this engine requires.

use std::sync::OnceLock;

use crate::utils::{board::Board, piece::{PieceColor, PieceType}};

/// Quantization factor the net was exported with.
const SCALE: i16 = 64;
/// Width of the accumulator.
const HIDDEN: usize = 256;

const NET_BYTES: &[u8] = include_bytes!("../../net.bin");

struct Net {
    /// 768 * HIDDEN input weights, feature major.
    input_weights: Vec<i16>,
    input_biases: [i16; HIDDEN],
    hidden_weights: [i16; HIDDEN],
    hidden_bias: i32,
}

/// Prophet orders its features pawn..king, whereas [`PieceType::to_index`]
/// returns the slot of the piece in sacre_dieu's bitboard array.
const PIECE_ORDER: [(PieceType, usize); 6] = [
    (PieceType::Pawn, 0),
    (PieceType::Knight, 1),
    (PieceType::Bishop, 2),
    (PieceType::Rook, 3),
    (PieceType::Queen, 4),
    (PieceType::King, 5),
];

fn net() -> &'static Net {
    static NET: OnceLock<Net> = OnceLock::new();
    NET.get_or_init(|| {
        let mut cursor = 0;
        let mut read_array = || {
            let len = u32::from_le_bytes(
                NET_BYTES[cursor..cursor + 4].try_into().expect("truncated net"),
            ) as usize;
            cursor += 4;

            let values: Vec<i16> = NET_BYTES[cursor..cursor + len * 2]
                .chunks_exact(2)
                .map(|pair| i16::from_le_bytes([pair[0], pair[1]]))
                .collect();
            cursor += len * 2;
            values
        };

        let input_weights = read_array();
        let input_biases = read_array();
        let hidden_weights = read_array();
        let hidden_biases = read_array();

        assert_eq!(input_weights.len(), 768 * HIDDEN, "unexpected input weight count");
        assert_eq!(input_biases.len(), HIDDEN, "unexpected accumulator width");
        assert_eq!(hidden_weights.len(), HIDDEN, "unexpected output weight count");

        Net {
            input_weights,
            input_biases: input_biases.try_into().expect("accumulator width mismatch"),
            hidden_weights: hidden_weights.try_into().expect("output width mismatch"),
            hidden_bias: hidden_biases[0] as i32,
        }
    })
}

/// Prophet's feature order is pawn..king, whereas [`PieceType::to_index`]
/// returns the piece's slot in sacre_dieu's bitboard array.
#[inline(always)]
fn feature_piece(piece: PieceType) -> usize {
    match piece {
        PieceType::Pawn => 0,
        PieceType::Knight => 1,
        PieceType::Bishop => 2,
        PieceType::Rook => 3,
        PieceType::Queen => 4,
        PieceType::King => 5,
    }
}

/// The incrementally updated first layer. This is the "efficiently updatable"
/// half of NNUE: a move touches at most a handful of features, so the
/// accumulator is patched by adding and subtracting those feature rows rather
/// than recomputed from all ~32 pieces.
#[derive(Clone)]
pub struct Accumulator {
    values: [i16; HIDDEN],
}

impl Accumulator {
    /// An accumulator holding an empty board, i.e. just the input biases.
    pub fn empty() -> Self {
        Accumulator { values: net().input_biases }
    }

    /// Recomputes the accumulator from scratch. Only needed when a position is
    /// set up from a FEN; play proceeds incrementally from there.
    pub fn refresh(&mut self, board: &Board) {
        self.values = net().input_biases;

        for (piece_type, _) in PIECE_ORDER {
            for color in [PieceColor::White, PieceColor::Black] {
                let mut bitboard = board.colored_piece(piece_type, color).board;

                while bitboard != 0 {
                    let square = bitboard.trailing_zeros() as usize;
                    bitboard &= bitboard - 1;
                    self.add(piece_type, color, square);
                }
            }
        }
    }

    #[inline(always)]
    fn weights(piece: PieceType, color: PieceColor, square: usize) -> &'static [i16] {
        let feature = (feature_piece(piece) * 2 + color.to_index()) * 64 + square;
        &net().input_weights[feature * HIDDEN..(feature + 1) * HIDDEN]
    }

    /// Adds a piece to the accumulator.
    #[inline(always)]
    pub fn add(&mut self, piece: PieceType, color: PieceColor, square: usize) {
        for (activation, weight) in self.values.iter_mut().zip(Self::weights(piece, color, square)) {
            *activation += weight;
        }
    }

    /// Removes a piece from the accumulator.
    #[inline(always)]
    pub fn remove(&mut self, piece: PieceType, color: PieceColor, square: usize) {
        for (activation, weight) in self.values.iter_mut().zip(Self::weights(piece, color, square)) {
            *activation -= weight;
        }
    }

    /// Runs the output layer, from the perspective of the side to move.
    pub fn eval(&self, black_to_move: bool) -> i32 {
        let net = net();

        let mut output = net.hidden_bias;
        for (activation, weight) in self.values.iter().zip(net.hidden_weights.iter()) {
            output += ((*activation).clamp(0, SCALE) as i32) * (*weight as i32);
        }

        let eval = output / (SCALE as i32 * SCALE as i32);

        if black_to_move { -eval } else { eval }
    }
}

/// Evaluates the board with the NNUE, from the perspective of the side to move.
pub fn evaluate_board(board: &Board) -> i32 {
    board.accumulator.eval(board.side_to_move == PieceColor::Black)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Values produced by the prophet crate itself, via `export_net`. If the
    /// board mapping here is wrong these will not line up.
    const REFERENCES: &[(&str, i32)] = &[
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 23),
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1", -23),
        ("r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3", -5),
        ("8/8/8/4k3/8/8/4KQ2/8 w - - 0 1", 1056),
        ("8/8/8/4k3/8/8/4KQ2/8 b - - 0 1", -1056),
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 56),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 42),
        ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 28),
    ];

    #[test]
    fn matches_prophet_reference_evaluations() {
        for (fen, expected) in REFERENCES {
            let board = Board::new(fen);
            assert_eq!(evaluate_board(&board), *expected, "mismatch on {fen}");
        }
    }

    /// Walks the whole move tree to a fixed depth and asserts that the
    /// incrementally updated accumulator is bit-for-bit identical to one
    /// rebuilt from scratch. This is what catches a mishandled castling rook,
    /// en passant capture, or promotion delta.
    fn assert_incremental_matches_refresh(board: &Board, depth: usize, line: &mut Vec<String>) {
        let mut fresh = Accumulator::empty();
        fresh.refresh(board);
        assert_eq!(
            board.accumulator.values, fresh.values,
            "accumulator drifted after {line:?}"
        );

        if depth == 0 {
            return;
        }

        let mut moves = crate::utils::piece_move::MoveArray::new();
        board.generate_moves(&mut moves, false);

        for piece_move in moves.iter() {
            if let Some(next) = board.make_move(piece_move, false) {
                line.push(format!("{piece_move:?}"));
                assert_incremental_matches_refresh(&next, depth - 1, line);
                line.pop();
            }
        }
    }

    #[test]
    fn incremental_updates_match_full_refresh() {
        // Positions chosen to exercise every delta path: castling both sides,
        // en passant, and a position where promotions (including capture
        // promotions) are available.
        const POSITIONS: &[&str] = &[
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2",
        ];

        for fen in POSITIONS {
            let board = Board::new(fen);
            assert_incremental_matches_refresh(&board, 3, &mut Vec::new());
        }
    }
}
