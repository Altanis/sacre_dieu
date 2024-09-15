use std::{sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender}, Arc}, time::{Duration, Instant}};
use arrayvec::ArrayVec;

use crate::{engine::search::{SearchEntry, Searcher}, utils::{board::Board, consts::{BEST_EVAL, DEEPEST_PROVEN_LOSS, DEEPEST_PROVEN_WIN, MAX_DEPTH, SHALLOWEST_PROVEN_LOSS, SHALLOWEST_PROVEN_WIN, WORST_EVAL}, piece::PieceColor, piece_move::{Move, MoveFlags, MoveSorter}}};

#[derive(Debug)]
pub enum UCICommands {
    SetPosition(String),
    ForceMove(String),
    NewGame,
    ResizeTT(usize),
    StartSearch(i64, i64, u64, u64, u64, u64, isize),
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
            let (
                mut time,
                mut depth,
                mut wtime,
                mut winc,
                mut btime,
                mut binc,
                mut nodes
            ) = (-1, -1, 0, 0, 0, 0, -1);

            while let Some(token) = args.next() {
                match token {
                    "infinite" => (time, depth, wtime, winc, btime, binc, nodes) = (-1, -1, 0, 0, 0, 0, -1),
                    "movetime" => time = args.next().expect("missing time argument").parse::<i64>().expect("failed to parse time argument"),
                    "depth" => depth = args.next().expect("missing depth argument").parse::<i64>().expect("failed to parse depth argument"),
                    "wtime" => wtime = args.next().expect("missing wtime arg").parse::<u64>().expect("failed to parse wtime"),
                    "btime" => btime = args.next().expect("missing btime argument").parse::<u64>().expect("failed to parse btime argument"),
                    "winc" => winc = args.next().expect("missing winc argument").parse::<u64>().expect("failed to parse winc argument"),
                    "binc" => binc = args.next().expect("missing binc argument").parse::<u64>().expect("failed to parse binc argument"),
                    "nodes" => nodes = args.next().expect("missing nodes argument").parse::<isize>().expect("failed to parse nodes argument"),
                    _ => {}
                }
            }

            // dbg!(time, depth, wtime, winc, btime, binc);
            sender.send(UCICommands::StartSearch(time, depth, wtime, winc, btime, binc, nodes)).expect("failed to send startsearch cmd");
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
    let mut searcher = Searcher::new(Duration::MAX, Duration::MAX, 5, stop_signal.clone());

    while let Ok(message) = receiver.recv() {
        match message {
            UCICommands::NewGame => {
                searcher.transposition_table.clear();
                searcher.past_boards.clear();
                searcher.search_stack = std::array::from_fn(|_| SearchEntry::default());
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
            UCICommands::StartSearch(time_limit, depth, white_time, winc, black_time, binc, max_nodes) => {
                stop_signal.store(false, Ordering::Relaxed);

                let engine_time_left = if board.side_to_move == PieceColor::White { white_time } else { black_time };
                let engine_inc_left = if board.side_to_move == PieceColor::White { winc } else { binc };
                
                let mut eval = 0;
                let mut timer: Instant = Instant::now();

                searcher.soft_tm = Duration::MAX;
                searcher.hard_tm = Duration::MAX;
                searcher.timer = Instant::now();
                searcher.max_depth = MAX_DEPTH;
                searcher.nodes = 0;
                searcher.best_move = None;

                if time_limit != -1 {
                    // Iterative deepening until time limit reached.
                    searcher.hard_tm = Duration::from_millis(time_limit as u64);
                    eval = searcher.search_timed(&board);
                } else if engine_time_left != 0 {
                    // Iterative deepening using soft and hard time limits.
                    searcher.soft_tm = Duration::from_millis(engine_time_left / 20 + engine_inc_left / 2);
                    searcher.hard_tm = Duration::from_millis(engine_time_left / 4);

                    eval = searcher.search_timed(&board);
                } else if depth != -1 {
                    // Search up to a specified depth.
                    searcher.max_depth = depth as usize;
                    eval = searcher.search_timed(&board);
                } else if max_nodes != -1 {
                    // Search up to a specified node count.
                    searcher.max_nodes = max_nodes;
                    eval = searcher.search_timed(&board);
                } else {
                    // Iterative deepening until `stop` is sent (or depth 127 is reached).
                    searcher.hard_tm = Duration::MAX;
                    eval = searcher.search_timed(&board);
                }

                let (timer, nodes, depth) = (searcher.timer, searcher.nodes, searcher.depth);

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
                        reply(&format!("info depth {} score mate {} time {} nodes {} nps {}", depth, mate_in, ms_time, nodes, nps));
                    } else if (DEEPEST_PROVEN_WIN..=SHALLOWEST_PROVEN_WIN).contains(&eval) {
                        let mate_in = (SHALLOWEST_PROVEN_WIN - eval) / 2;
                        reply(&format!("info depth {} score mate {} time {} nodes {} nps {}", depth, mate_in, ms_time, nodes, nps));
                    } else {
                        reply(&format!("info depth {} score cp {} time {} nodes {} nps {}", depth, eval, ms_time, nodes, nps));
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