use colored::Colorize;
use std::time::Instant;

use crate::utils::{
    board::Board,
    consts::{DEEPEST_PROVEN_LOSS, DEEPEST_PROVEN_WIN, SHALLOWEST_PROVEN_LOSS, SHALLOWEST_PROVEN_WIN},
    piece::{PieceColor, PieceType},
    piece_move::Move,
};

#[inline]
fn rainbow<S: AsRef<str>>(s: S, i: usize) -> colored::ColoredString {
    match i % 6 {
        0 => s.as_ref().red(),
        1 => s.as_ref().yellow(),
        2 => s.as_ref().green(),
        3 => s.as_ref().cyan(),
        4 => s.as_ref().blue(),
        _ => s.as_ref().magenta(),
    }
}

pub fn print_search_progress(depth: usize, score: i32, nodes: usize, timer: &Instant, best_move: Option<Move>) {
    let elapsed_ms = timer.elapsed().as_millis();
    let nps = if elapsed_ms > 0 { (nodes as u128 * 1000 / elapsed_ms) as u64 } else { 0 };

    let score_str = if (SHALLOWEST_PROVEN_LOSS..=DEEPEST_PROVEN_LOSS).contains(&score) {
        let mate_in = (SHALLOWEST_PROVEN_LOSS - score + 1) / 2;
        format!("Mate {}", -mate_in).red().bold()
    } else if (DEEPEST_PROVEN_WIN..=SHALLOWEST_PROVEN_WIN).contains(&score) {
        let mate_in = (SHALLOWEST_PROVEN_WIN - score + 1) / 2;
        format!("Mate {}", mate_in).green().bold()
    } else {
        format!("{:.2}", score as f32 / 100.0).cyan().bold()
    };

    let best_move_str = best_move.map(|m| m.to_uci()).unwrap_or_else(|| "none".to_string());

    println!(
        "{} {} {} {:>9} {} {:>9} {} {:>8} {} {:>5} {} {}",
        rainbow(" depth", 0),
        format!("{:>2}", depth).bold(),
        rainbow("| score", 1),
        score_str,
        rainbow("| nodes", 2),
        format!("{:>9}", nodes).bold(),
        rainbow("| nps", 3),
        format!("{:>8}", nps).bold(),
        rainbow("| time", 4),
        format!("{:>5}ms", elapsed_ms).bold(),
        rainbow("| pv", 5),
        best_move_str.yellow().bold()
    );
}

fn get_piece_char(piece_type: PieceType, color: PieceColor) -> &'static str {
    match color {
        PieceColor::White => match piece_type {
            PieceType::Pawn => "♙", PieceType::Knight => "♘", PieceType::Bishop => "♗",
            PieceType::Rook => "♖", PieceType::Queen => "♕", PieceType::King => "♔",
        },
        PieceColor::Black => match piece_type {
            PieceType::Pawn => "♟", PieceType::Knight => "♞", PieceType::Bishop => "♝",
            PieceType::Rook => "♜", PieceType::Queen => "♛", PieceType::King => "♚",
        },
    }
}

pub fn print_pretty_board(board: &Board) {
    println!();
    for rank in (0..8).rev() {
        print!(" {} ", (rank + 1).to_string().dimmed());
        for file in 0..8 {
            let piece = &board.board[(rank * 8 + file) as usize];

            let piece_str = if let Some(p) = piece {
                let glyph = get_piece_char(p.piece_type, p.piece_color);
                match p.piece_color {
                    PieceColor::White => glyph.white().bold(),
                    PieceColor::Black => glyph.black().bold(),
                }
            } else {
                " ".normal()
            };

            print!(" {} ", piece_str);
        }
        println!();
    }

    let files = ('a'..='h').map(|c| format!(" {} ", c)).collect::<String>().dimmed();
    println!("    {}", files);

    let side = if board.side_to_move == PieceColor::White { "White".bold().white() } else { "Black".bold().dimmed() };
    println!("\n{} {}", "Turn:".yellow(), side);
    println!("{} {}", "Zobrist:".yellow(), format!("{:x}", board.zobrist_key).dimmed());
}
