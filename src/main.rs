#![feature(let_chains)]
#![feature(string_remove_matches)]


use state::consts::{BISHOP_MAGICS, BISHOP_MASKS, ROOK_MAGICS};

mod state;

/// Renders a chess board for debugging purposes.
fn render_board(board: &state::board::Board) {
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
    std::env::set_var("RUST_BACKTRACE", "1");

    let board = state::board::Board::new("8/6k2/8/8/8/8/8/8 w - - 0 1");

    // let mut file = std::fs::File::create("rook.txt").expect("couldnt make file");
    // let mut shit = "[".to_string();
    // for bitboard in board.sliding_rook_bitboard.iter() {
    //     let mut str = bitboard.serialize(serde_json::value::Serializer).expect("couldnt serialize").to_string();
    //     str.remove_matches('"');

    //     shit += format!("{}, ", str).as_str();
    // }

    // shit += "]";

    // file.write_all(shit.as_bytes()).expect("couldnt write");

    let position = state::piece::Position::from_code("g8");

    let mut blocker_bitboard = state::board::Bitboard::new(0);
    blocker_bitboard.set_bit(state::piece::Position::from_code("d5"));
    blocker_bitboard.render_bitboard(position);

    let magic = &BISHOP_MAGICS[position.square()];
    let bitboard = BISHOP_MASKS[state::board::Board::generate_magic_index(magic, &blocker_bitboard)];
    bitboard.render_bitboard(position);

    // let position = state::piece::Position::from_code("g8");
    // let time = std::time::Instant::now();
    // println!("{}", position.is_under_attack(&board, state::piece::PieceColor::Black));
    // dbg!(time.elapsed());
}