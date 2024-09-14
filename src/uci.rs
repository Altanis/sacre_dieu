use std::{sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc}, time::{Duration, Instant}};
use arrayvec::ArrayVec;

use crate::{engine::search::Searcher, utils::{board::Board, consts::{BEST_EVAL, DEEPEST_PROVEN_LOSS, DEEPEST_PROVEN_WIN, SHALLOWEST_PROVEN_LOSS, SHALLOWEST_PROVEN_WIN, WORST_EVAL}, piece::PieceColor, piece_move::{Move, MoveFlags, MoveSorter}}};

#[derive(Debug)]
pub enum UCICommands {
    SetPosition(String),
    ForceMove(String),
    NewGame,
    ResizeTT(usize),
    StartSearch(i64, i64, u64, u64), // todo move stop_signal to handle_search function
    PrintBoard
}

pub fn handle_command(command: &str, sender: Sender<UCICommands>, stop_signal: Arc<AtomicBool>) {
    let mut args = command.split(' ');
    let command = args.next().expect("received empty UCI command");

    match command {
        "uci" => reply("uciok"),
        "isready" => reply("readyok"),
        "setoption" => {
            if args.next() == Some("name") && args.next() == Some("Hash") && args.next() == Some("value") {
                let size = args.next().expect("failed to parse hash size").parse::<usize>().expect("failed to parse hash size");
                sender.send(UCICommands::ResizeTT(size)).expect("failed to send resize cmd");
            } else {
                reply("setoption only supports setting hashsize.");
            }
        },
        "ucinewgame" => {
            sender.send(UCICommands::NewGame).expect("couldnt send ucinewgame");
            stop_signal.store(true, Ordering::Relaxed);
        },
        "stop" => stop_signal.store(true, Ordering::Relaxed),
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
                    "infinite" => sender.send(UCICommands::StartSearch(-1, -1, 0, 0)).expect("failed to send startsearch cmd"),
                    "movetime" => {
                        let time = args.next().expect("missing time argument").parse::<i64>().expect("failed to parse time argument");
                        sender.send(UCICommands::StartSearch(time, -1, 0, 0)).expect("failed to send startsearch cmd");
                    },
                    "depth" => {
                        let depth = args.next().expect("missing depth argument").parse::<i64>().expect("failed to parse depth argument");
                        sender.send(UCICommands::StartSearch(-1, depth, 0, 0)).expect("failed to send startsearch cmd");
                    },
                    "wtime" => {
                        let white_ms_time = args.next().expect("missing wtime arg").parse::<u64>().expect("failed to parse wtime");
                        let _ = args.next().expect("missing btime label");
                        let black_ms_time = args.next().expect("missing btime arg").parse::<u64>().expect("failed to parse btime");

                        sender.send(UCICommands::StartSearch(-1, -1, white_ms_time, black_ms_time)).expect("wtime invalid");
                    },
                    "btime" => {
                        let black_ms_time = args.next().expect("missing btime arg").parse::<u64>().expect("failed to parse btime");
                        let _ = args.next().expect("missing wtime label");
                        let white_ms_time = args.next().expect("missing wtime arg").parse::<u64>().expect("failed to parse wtime");

                        sender.send(UCICommands::StartSearch(-1, -1, white_ms_time, black_ms_time)).expect("btime invalid");
                    },
                    _ => sender.send(UCICommands::StartSearch(-1, 5, 0, 0)).expect("failed to send startsearch cmd")
                }
            } else {
                println!("provide an argument (infinite/movetime/depth/wtime/btime)");
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

pub fn handle_board(receiver: Receiver<UCICommands>, stop_signal: Arc<AtomicBool>) {
    let mut board = Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut searcher = Searcher::new(Duration::MAX, 5, stop_signal.clone());

    while let Ok(message) = receiver.recv() {
        match message {
            UCICommands::NewGame => {
                searcher.transposition_table.clear();
                searcher.past_boards.clear();
                searcher.move_sorter = MoveSorter::new();
            },
            UCICommands::SetPosition(pos) => board = Board::new(pos.as_str()),
            UCICommands::ForceMove(moves) => {
                for uci_move in moves.split(' ') {
                    let piece_move = Move::from_uci(uci_move);
                    
                    if piece_move.flags == MoveFlags::None {
                        let piece = board.board[piece_move.initial.index()].as_ref().expect("no piece on initial move index");
                    
                        let mut moves = ArrayVec::new();
                        piece.generate_moves(&board, piece_move.initial, false, &mut moves);

                        let real_move = moves
                            .iter()
                            .find(|mv| mv.end.index() == piece_move.end.index())
                            .expect("couldnt find move");

                        board = board.make_move(real_move, false).unwrap();
                    } else {
                        board = board.make_move(&piece_move, false).unwrap();
                    }
                }
            },
            UCICommands::ResizeTT(mb) => {
                searcher.transposition_table.resize_mb(mb);
            },
            UCICommands::StartSearch(time_limit, depth, white_time, black_time) => {
                stop_signal.store(false, Ordering::Relaxed);

                let engine_time_left = if board.side_to_move == PieceColor::White { white_time } else { black_time };
                
                let mut eval = 0;
                let mut timer: Instant = Instant::now();
                let (mut nodes, mut max_depth) = (0, 0);

                searcher.time_limit = Duration::MAX;
                searcher.timer = Instant::now();
                searcher.max_depth = 5;
                searcher.nodes = 0;
                searcher.best_move = None;

                let (alpha, beta) = (WORST_EVAL, BEST_EVAL);

                if time_limit != -1 {
                    // Iterative deepening until time limit reached.
                    searcher.time_limit = Duration::from_millis(time_limit as u64);
                    eval = searcher.search_timed(&board);
                } else if engine_time_left != 0 {
                    // hard limit by dividing engine_time_left by 20-30, then by using linreg, then softlimit; sprt against eachother
                    searcher.time_limit = Duration::from_millis(engine_time_left / 20);
                    eval = searcher.search_timed(&board);
                } else if depth != -1 {
                    // Search up to a specified depth.
                    searcher.max_depth = depth as usize;
                    eval = searcher.search::<true>(&board, searcher.max_depth, 0, alpha, beta);
                } else {
                    // Iterative deepening until `stop` is sent.
                    searcher.time_limit = Duration::MAX;
                    eval = searcher.search_timed(&board);
                }

                (timer, nodes, max_depth) = (searcher.timer, searcher.nodes, searcher.max_depth);

                let ms_time = timer.elapsed().as_millis();
                let nps = nodes as f64 / (ms_time as f64 / 1000.0);

                if let Some(best_move) = searcher.best_move {
                    board = board.make_move(&best_move, false).unwrap();

                    if board.half_move_counter == 0 {
                        searcher.past_boards.clear();
                    }
                    
                    searcher.past_boards.push(board.zobrist_key);

                    if (SHALLOWEST_PROVEN_LOSS..=DEEPEST_PROVEN_LOSS).contains(&eval) {
                        let mate_in = (SHALLOWEST_PROVEN_LOSS - eval) / 2;
                        reply(&format!("info depth {} score mate {} time {} nodes {} nps {}", max_depth, mate_in, ms_time, nodes, nps));
                    } else if (DEEPEST_PROVEN_WIN..=SHALLOWEST_PROVEN_WIN).contains(&eval) {
                        let mate_in = (SHALLOWEST_PROVEN_WIN - eval) / 2;
                        reply(&format!("info depth {} score mate {} time {} nodes {} nps {}", max_depth, mate_in, ms_time, nodes, nps));
                    } else {
                        reply(&format!("info depth {} score cp {} time {} nodes {} nps {}", max_depth, eval, ms_time, nodes, nps));
                    }

                    reply(&format!("bestmove {}", best_move.to_uci()));
                } else {
                    panic!("null move");
                }
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