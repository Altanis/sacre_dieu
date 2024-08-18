#![feature(let_chains)]

mod state;

/// Renders a chess board for debugging purposes.
fn render_board(board: state::board::Board) {
    for row in board.board.iter() {
        for cell in row.iter() {
            print!("| ");
            match cell {
                Some(piece) => {
                    let uppercase = piece.piece_color == state::piece::PieceColor::White;
                    match piece.piece_type {
                        state::piece::PieceType::Pawn => if uppercase { print!("P") } else { print!("p") },
                        state::piece::PieceType::Rook => if uppercase { print!("R") } else { print!("r") },
                        state::piece::PieceType::Knight => if uppercase { print!("N") } else { print!("n") },
                        state::piece::PieceType::Bishop => if uppercase { print!("B") } else { print!("b") },
                        state::piece::PieceType::Queen => if uppercase { print!("Q") } else { print!("q") },
                        state::piece::PieceType::King => if uppercase { print!("K") } else { print!("k") },
                    }
                },
                None => print!(" "),
            }
        }

        println!("|");
    }
}

fn main() {
    let board = state::board::Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    let position = state::piece::Position::from_code("a1").expect("bruh");
    let mut bitboard_idx = 0;

    board.bishop_blockers
        [(position.rank * 8 + position.file) as usize]
        [bitboard_idx]
        .render_bitboard(position);

    board.sliding_bishop_bitboard
        [(position.rank * 8 + position.file) as usize]
        [bitboard_idx]
        .render_bitboard(position);

    // render_attack_bitboard(
    //     position,
    //     board.attack_bitboard
    //         [state::piece::PieceType::Pawn.to_index() - 1]
    //         [(position.rank * 8 + position.file) as usize]
    //         .clone()
    // );

    // println!();

    // render_attack_bitboard(
    //     position,
    //     board.attack_bitboard
    //         [0]
    //         [(position.rank * 8 + position.file) as usize]
    //         .clone()
    // );
}
