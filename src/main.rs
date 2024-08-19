#![feature(let_chains)]

use state::piece::Position;

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

    let position = state::piece::Position::from_code("e4").expect("bruh");
    let square = (position.rank * 8 + position.file) as usize;

    let mut blocker_bitboard = state::board::Bitboard::new(0);
    // blocker_bitboard.set_bit(state::piece::Position::from_code("e3").unwrap());
    // blocker_bitboard.set_bit(state::piece::Position::from_code("g4").unwrap());
    // blocker_bitboard.set_bit(state::piece::Position::from_code("b4").unwrap());
    // blocker_bitboard.set_bit(Position::from_code("d5").unwrap());
    // blocker_bitboard.set_bit(Position::from_code("f5").unwrap());
    // blocker_bitboard.set_bit(Position::from_code("g6").unwrap());
    // blocker_bitboard.set_bit(Position::from_code("f3").unwrap());
    // blocker_bitboard.set_bit(Position::from_code("c2").unwrap());

    blocker_bitboard.render_bitboard(position);

    board.sliding_bishop_bitboard[square]
    [state::board::Board::generate_magic_index(&blocker_bitboard, position, state::piece::PieceType::Bishop) as usize]
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
