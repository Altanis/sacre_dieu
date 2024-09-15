import chess
import chess.pgn
import io
import re

starting_fen = "r1bqk2r/p2nppb1/2pp1np1/1p5p/3PP3/P1N3PP/1PP1NPB1/R1BQK2R w KQkq - 0 1"
pgn = """
[Event "?"]
[Site "?"]
[Date "2024.09.15"]
[Round "1"]
[White "SacreDieu-base"]
[Black "SacreDieu-dev"]
[Result "0-1"]
[FEN "r1bqk2r/p2nppb1/2pp1np1/1p5p/3PP3/P1N3PP/1PP1NPB1/R1BQK2R w KQkq - 0 1"]
[GameDuration "00:00:23"]
[GameEndTime "2024-09-15T07:19:00.517 UTC"]
[GameStartTime "2024-09-15T07:18:37.394 UTC"]
[PlyCount "62"]
[SetUp "1"]
[Termination "stalled connection"]
[TimeControl "3.39+0.03"]

1. e5 {+1.37 14/0 267 458718} Nd5 {-1.56 14/0 227 466783}
2. Nxd5 {+1.42 15/0 307 630019} cxd5 {-1.39 14/0 183 310190}
3. Bxd5 {+1.38 15/0 396 786825} Rb8 {-1.32 13/0 211 306333}
4. e6 {+1.28 14/0 169 296474} fxe6 {-1.28 13/0 190 422934}
5. Bxe6 {+1.27 13/0 303 493259} h4 {-1.17 13/0 387 829250}
6. Nf4 {+1.31 11/0 228 320914} Nf8 {-1.23 13/0 208 511803}
7. Bxc8 {+1.00 11/0 263 408931} Qxc8 {-0.96 12/0 161 374768}
8. d5 {+0.98 11/0 159 231127} hxg3 {-0.76 12/0 149 362967}
9. fxg3 {+0.90 10/0 107 158793} Be5 {-0.70 12/0 141 330729}
10. Kf2 {+0.43 10/0 123 179225} Bxf4 {-0.56 13/0 135 332092}
11. Bxf4 {+0.59 13/0 105 234380} Rxh3 {-0.59 12/0 113 227712}
12. Qd3 {+0.71 11/0 90 130794} Rxh1 {-0.57 12/0 125 182507}
13. Rxh1 {+0.60 13/0 79 115649} Qc4 {-0.52 13/0 125 180046}
14. Re1 {+0.57 12/0 118 170047} Qxd3 {-0.36 13/0 94 216092}
15. cxd3 {+0.47 13/0 72 183665} Kf7 {-0.34 13/0 111 180058}
16. Rc1 {+0.31 11/0 98 148500} Rb7 {-0.31 14/0 119 299290}
17. Bd2 {+0.28 13/0 127 200800} Nd7 {-0.22 14/0 104 156697}
18. Ke3 {+0.29 12/0 70 104704} Kf6 {-0.03 13/0 161 383456}
19. Rf1+ {+0.31 11/0 65 146864} Ke5 {+0.04 17/0 77 204000}
20. Rf7 {0.00 15/0 68 182853} Nf6 {+0.05 17/0 88 244650}
21. Bc3+ {-0.04 16/0 78 227345} Kxd5 {+0.07 19/0 95 280966}
22. Bxf6 {-0.01 18/0 68 203522} Ke6 {-0.02 21/0 65 187509}
23. Rxe7+ {-0.01 20/0 64 121370} Rxe7 {-0.09 22/0 66 125725}
24. Bxe7 {-0.02 22/0 72 134566} Kxe7 {-0.08 23/0 66 127625}
25. Kd4 {+0.06 20/0 65 120670} Ke6 {-0.09 20/0 56 106058}
26. g4 {+0.05 21/0 51 97205} a5 {-0.11 20/0 53 99700}
27. b3 {+0.24 22/0 82 164444} g5 {+0.17 23/0 55 112155}
28. a4 {-0.15 23/0 49 100087} b4 {+0.17 26/0 49 93489}
29. Ke4 {-0.15 26/0 129 210535} d5+ {+0.17 64/0 55 133059}
30. Kd4 {-0.15 68/0 43 101351} Kd6 {+0.17 103/0 155 439500}
31. Ke3 {0.00 128/0 32 113757}
Ke5 {+0.17 101/0 124 300128}
"""

nodes = []
move_info = []

def find_nodes(pgn: str):
    _, per_move_pgn = pgn.strip().split("\n\n")
    for move in per_move_pgn.strip().split("\n"):
        print(move)
        _, real_move_str = move.split(". ")
        movedata = re.findall(r'[^{} ]+|\{[^}]+\}', real_move_str.strip())

        if len(movedata) == 4:
            move1_nodes = movedata[1].split(" ")[3]
            move2_nodes = movedata[3].split(" ")[3]

            nodes.append(int(move1_nodes[:-1]))
            nodes.append(int(move2_nodes[:-1]))
        elif len(movedata) == 2:
            move1_nodes = movedata[1].split(" ")[3]
            nodes.append(int(move1_nodes[:-1]))
        else:
            print("disaster")


def pgn_to_uci_command(starting_fen: str, pgn: str) -> str:
    find_nodes(pgn)

    board = chess.Board(starting_fen)
    cleaned_pgn = pgn
    pgn_io = io.StringIO(cleaned_pgn)
    
    print("Reading PGN...")
    game = chess.pgn.read_game(pgn_io)
    
    if game is None:
        raise ValueError("Failed to parse PGN")
    
    moves = []
    node = game
    i = 0

    while node.variations:
        next_node = node.variation(0)

        move = next_node.move
        if move not in board.legal_moves:
            raise ValueError(f"Illegal move {move} for the current board position")
        moves.append(move.uci())
        move_info.append({ "move": move.uci(), "nodes": nodes[i] })
        
        board.push(move)
        node = next_node
        i += 1
    
    uci_command = f"position fen {starting_fen} moves {' '.join(moves)}"
    
    return uci_command

def moves_to_nodes() -> str:
    uci_command = ""

    for (i, entry) in enumerate(move_info):
        uci_command += f"position fen {starting_fen} moves"
        for prev_idx in range(0, i + 1):
            uci_command += f" {move_info[prev_idx]["move"]}"
        uci_command += f"\ngo nodes {entry["nodes"]}\n"

    return uci_command

uci_command = pgn_to_uci_command(starting_fen, pgn)
# print(uci_command)
print(moves_to_nodes())
