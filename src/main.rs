#![feature(let_chains)]
#![feature(string_remove_matches)]

mod state;

/// Renders a chess board for debugging purposes.
fn render_board(board: &state::board::Board) {

}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    
    // let board = state::board::Board::new("rnbqk1nr/pppppp1p/6pb/8/8/3P4/PPPKPPPP/RN1Q1BNR w kq - 0 3");
    // let tile = Position::from_code("d2");
    // dbg!(tile.is_under_attack(&board, state::piece::PieceColor::Black));

    // let board = state::board::Board::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let board = state::board::Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // dbg!(board.perft(3, 3, &mut vec![]));

    let time = std::time::Instant::now();
    let nodes = board.pure_perft(5);
    let time = time.elapsed();
    println!("Time: {:?}, Nodes: {}, NPS: {}", time, nodes, (nodes as f64) / time.as_secs_f64());

    // let tile = Position::from_code("a7");
    // Bitboard::new(BLACK_PAWN_MASK[tile.square()].0).render_bitboard(pos);

    // let mut file = std::fs::File::create("rook.txt").expect("couldnt make file");
    // let mut shit = "[".to_string();
    // for bitboard in board.sliding_rook_bitboard.iter() {
    //     let mut str = bitboard.serialize(serde_json::value::Serializer).expect("couldnt serialize").to_string();
    //     str.remove_matches('"');

    //     shit += format!("{}, ", str).as_str();
    // }

    // shit += "]";

    // file.write_all(shit.as_bytes()).expect("couldnt write");

    // let position = state::piece::Position::from_code("c1");

    // let mut blocker_bitboard = state::board::Bitboard::new(18446462598732906495);
    // // blocker_bitboard.set_bit(state::piece::Position::from_code("d5"));
    // blocker_bitboard.render_bitboard(position);

    // let magic = &BISHOP_MAGICS[position.square()];
    // let bitboard = get_bishop_mask(state::board::Board::generate_magic_index(magic, &blocker_bitboard));
    // bitboard.render_bitboard(position);

    // let position = state::piece::Position::from_code("g8");
    // let time = std::time::Instant::now();
    // println!("{}", position.is_under_attack(&board, state::piece::PieceColor::Black));
    // dbg!(time.elapsed());
}