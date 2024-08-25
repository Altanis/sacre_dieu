use std::sync::mpsc::{Receiver, Sender};
use crate::utils::{board::Board, consts::{I32_NEGATIVE_INFINITY, I32_POSITIVE_INFINITY}, piece::Move};

#[derive(Debug)]
pub enum UCICommands {
    SetPosition(String),
    ForceMove(String),
    StartSearch(i64, i64),
    StopSearch,
    Reset,
    PrintBoard
}

pub fn handle_command(command: &str, sender: Sender<UCICommands>) {
    let mut args = command.split(' ');
    let command = args.next().expect("received empty UCI command");

    match command {
        "uci" => reply("uciok"),
        "isready" => reply("readyok"),
        "ucinewgame" => sender.send(UCICommands::Reset).expect("failed to send reset cmd"),
        "stop" => sender.send(UCICommands::StopSearch).expect("failed to send stopsearch cmd"),
        "position" => {
            let tokens: Vec<&str> = args.collect();

            // do later
            let mut moves_index = 0;

            if tokens[0] == "startpos" {
                if tokens.get(1) == Some(&"moves") {
                    moves_index = 1;
                }

                sender.send(UCICommands::SetPosition("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string())).expect("failed to send position cmd (startpos)");
            } else if tokens[0] == "fen" {
                let mut fen = String::new();

                for (i, token) in tokens.iter().enumerate() {
                    if *token == "moves" {
                        moves_index = i;
                        break;
                    } else if *token == "fen" {
                        continue;
                    }

                    fen += token;
                    fen += " ";
                }

                sender.send(UCICommands::SetPosition(fen)).expect("failed to send position cmd (fen)");
            } else {
                panic!("expected startpos/fen as initial token");
            }

            if moves_index != 0 {
                sender.send(UCICommands::ForceMove(tokens[moves_index + 1..].join(" "))).expect("failed to send force move cmd");
            }
        },
        "go" => {
            if let Some(token) = args.next() {
                match token {
                    "infinite" => sender.send(UCICommands::StartSearch(-1, -1)).expect("failed to send startsearch cmd"),
                    "movetime" => {
                        let time = args.next().expect("missing time argument").parse::<i64>().expect("failed to parse time argument");
                        sender.send(UCICommands::StartSearch(time, -1)).expect("failed to send startsearch cmd");
                    },
                    "depth" => {
                        let depth = args.next().expect("missing depth argument").parse::<i64>().expect("failed to parse depth argument");
                        sender.send(UCICommands::StartSearch(-1, depth)).expect("failed to send startsearch cmd");
                    },
                    _ => sender.send(UCICommands::StartSearch(-1, 5)).expect("failed to send startsearch cmd")
                }
            }
        },
        "d" => sender.send(UCICommands::PrintBoard).expect("failed to send printboard cmd"),
        "quit" => {
            println!("asked to quit");
            std::process::exit(0);
        }
        _ => {}
    }
}

pub fn handle_board(receiver: Receiver<UCICommands>) {
    let mut board = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut should_stop = false;
    let mut infinite_search_running = false;

    while let Ok(message) = receiver.recv() {
        match message {
            UCICommands::SetPosition(pos) => board = Board::new(pos.as_str()),
            UCICommands::ForceMove(moves) => {
                for mv in moves.split(' ') {
                    let mv = Move::from_uci(mv);
                    board = board.make_move(&mv).unwrap();
                }
            },
            UCICommands::StartSearch(time, depth) => {
                // if depth = -1, search until time is up
                // if time = -1, search until depth is reached
                // if both -1, infinite search till stop

                // todo poll `should_stop` flag in between iterations
                
                let depth = 5;

                let time = std::time::Instant::now();
                let (eval, nodes, best_move) = board.search(depth, I32_NEGATIVE_INFINITY, I32_POSITIVE_INFINITY);
                let ms_time = time.elapsed().as_millis();

                let nps = nodes as f64 / (ms_time as f64 / 1000.0);

                if let Some(best_move) = best_move {
                    board = board.make_move(&best_move).unwrap();
                    reply(&format!("info depth {} score cp {} time {} nodes {} nps {}", depth, eval, ms_time, nodes, nps));
                    reply(&format!("bestmove {}", best_move.to_uci()));
                } else {
                    panic!("null move");
                }
            },
            UCICommands::StopSearch => {
                should_stop = true;
            },
            UCICommands::Reset => {
                should_stop = true;
                // reset transposition table
            },
            UCICommands::PrintBoard => {
                dbg!(&board);
            }
        }
    }
}

pub fn reply(response: &str) {
    println!("{}", response);
}