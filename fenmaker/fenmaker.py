import chess
import chess.pgn
import io
import re

def clean_pgn(pgn: str) -> str:
    """Remove comments and annotations from PGN."""
    cleaned_pgn = re.sub(r"\{[^}]*\}", "", pgn)
    return cleaned_pgn.strip()

def pgn_to_uci_command(starting_fen: str, pgn: str) -> str:
    # Load the starting FEN into a board
    board = chess.Board(starting_fen)
    
    # Clean the PGN to remove comments
    cleaned_pgn = clean_pgn(pgn)
    
    # Use StringIO to create a file-like object from the cleaned PGN string
    pgn_io = io.StringIO(cleaned_pgn)
    
    # Read the PGN
    print("Reading PGN...")
    game = chess.pgn.read_game(pgn_io)
    
    if game is None:
        raise ValueError("Failed to parse PGN")
    
    # Extract the moves from the PGN
    moves = []
    node = game
    while node.variations:
        next_node = node.variation(0)
        move = next_node.move
        if move not in board.legal_moves:
            raise ValueError(f"Illegal move {move} for the current board position")
        moves.append(move.uci())  # Use UCI notation directly for the moves
        board.push(move)  # Update the board
        node = next_node
    
    # Create the UCI command
    uci_command = f"position fen {starting_fen} moves {' '.join(moves)}"
    
    return uci_command

# Example usage
starting_fen = "rnbq1knr/pp3ppp/4p3/2bpP3/5BQ1/2N5/PPP2PPP/R3KBNR w KQ - 0 1"
pgn = """
[Event "?"]
[Site "?"]
[Date "2024.09.14"]
[Round "1"]
[White "SacreDieu-dev"]
[Black "SacreDieu-base"]
[Result "0-1"]
[FEN "rnbq1knr/pp3ppp/4p3/2bpP3/5BQ1/2N5/PPP2PPP/R3KBNR w KQ - 0 1"]
[GameDuration "00:00:25"]
[GameEndTime "2024-09-14T22:52:56.517 UTC"]
[GameStartTime "2024-09-14T22:52:30.708 UTC"]
[PlyCount "126"]
[SetUp "1"]
[Termination "stalled connection"]
[TimeControl "3.37+0.03"]

1. Na4 {+0.60 10/0 189 246501} Bb6 {-0.62 12/0 191 361829}
2. Nf3 {+0.68 11/0 202 274102} Nc6 {-0.68 12/0 305 434783}
3. Bd3 {+0.58 12/0 243 351099} Nge7 {-0.54 13/0 293 447916}
4. O-O {+0.51 12/0 323 835324} Ng6 {-0.52 13/0 258 698097}
5. Rae1 {+0.44 10/0 190 508365} Kg8 {-0.50 11/0 161 254847}
6. a3 {+0.49 10/0 328 515481} h5 {-0.49 12/0 133 208571}
7. Qg5 {+0.47 12/0 159 249036} Qxg5 {-0.52 13/0 184 289578}
8. Bxg5 {+0.42 13/0 116 190759} Bc7 {-0.31 14/0 140 313711}
9. Bxg6 {+0.13 13/0 144 231373} fxg6 {-0.15 13/0 132 214151}
10. b4 {+0.13 13/0 107 173334} b5 {-0.20 15/0 112 191160}
11. Nc5 {+0.14 14/0 110 181767} a5 {-0.34 15/0 182 302035}
12. c3 {+0.10 14/0 135 233756} Kh7 {-0.20 12/0 108 173969}
13. Nd4 {+0.13 12/0 254 385101} Nxd4 {-0.14 14/0 96 161298}
14. cxd4 {+0.13 13/0 115 186094} Re8 {-0.13 13/0 106 212303}
15. Be3 {+0.12 11/0 80 130739} Bb6 {-0.12 14/0 103 209414}
16. Rc1 {+0.17 14/0 109 266029} axb4 {-0.09 15/0 88 146766}
17. axb4 {+0.14 16/0 115 197372} Ra2 {-0.16 15/0 128 215342}
18. Rfd1 {+0.20 16/0 98 240894} Kg8 {-0.17 13/0 112 286622}
19. Rd2 {+0.23 14/0 67 116347} Rxd2 {-0.23 14/0 112 217535}
20. Bxd2 {+0.14 14/0 155 379146} Re7 {-0.16 13/0 72 186417}
21. Ra1 {+0.21 13/0 108 286903} Ra7 {-0.24 15/0 78 218463}
22. Rxa7 {+0.34 18/0 66 197604} Bxa7 {-0.36 19/0 78 233943}
23. h4 {+0.33 17/0 85 158149} Kf7 {-0.33 18/0 98 194412}
24. Kf1 {+0.34 17/0 46 138165} Bb6 {-0.37 18/0 96 176701}
25. Bg5 {+0.37 17/0 53 158421} Ke8 {-0.34 20/0 54 144352}
26. Ke2 {+0.34 19/0 65 170050} Bd7 {-0.34 21/0 55 108575}
27. Ke3 {+0.39 18/0 45 124254} Bc7 {-0.35 17/0 55 100442}
28. g3 {+0.34 17/0 46 121290} Bb6 {-0.35 22/0 58 162312}
29. Kf3 {+0.39 17/0 59 158407} Bc7 {-0.33 16/0 58 102985}
30. Kf4 {+0.34 18/0 39 113197} Bb6 {-0.34 20/0 53 158368}
31. Ke3 {+0.35 20/0 57 168276} Bc7 {-0.33 20/0 47 109787}
32. Kf4 {+0.35 21/0 40 127151} Bb6 {-0.33 22/0 56 105175}
33. Ke3 {+0.35 21/0 36 73385} Bc7 {-0.33 20/0 44 86586}
34. Kf3 {+0.35 20/0 37 75624} Bb6 {-0.33 20/0 57 105223}
35. Kf4 {+0.34 19/0 43 81522} Bc7 {-0.34 21/0 62 111686}
36. Kf3 {+0.31 17/0 37 68732} Bb6 {-0.37 20/0 43 79566}
37. Kf4 {+0.35 18/0 40 72632} Ba7 {-0.37 23/0 56 171230}
38. Ke3 {+0.36 19/0 41 123968} Bb6 {-0.37 22/0 106 269040}
39. Kd3 {+0.33 18/0 45 83366} Bc7 {-0.31 19/0 46 87117}
40. Be3 {+0.31 18/0 37 118123} Bb6 {-0.23 20/0 34 107952}
41. Bg5 {+0.32 18/0 41 126242} Bd8 {-0.23 21/0 48 145220}
42. Be3 {+0.26 18/0 62 185458} Be7 {-0.24 21/0 83 241977}
43. Ke2 {+0.24 17/0 29 85916} Kd8 {-0.24 19/0 29 90456}
44. Bf4 {+0.28 16/0 42 122470} Kc7 {-0.24 17/0 30 92885}
45. Kf3 {+0.24 18/0 32 97161} Bd8 {-0.24 18/0 31 93158}
46. Bg5 {+0.24 17/0 31 89628} Bxg5 {-0.23 23/0 28 86437}
47. hxg5 {+0.23 23/0 28 87049} Kd8 {-0.23 26/0 31 60557}
48. Ke3 {+0.23 26/0 32 61181} Ke7 {-0.23 30/0 36 111197}
49. Nxd7 {+0.23 32/0 32 64617} Kxd7 {-0.26 43/0 44 143903}
50. Kf4 {+0.27 49/0 40 145495} Ke7 {-0.26 47/0 28 113095}
51. Ke3 {+0.26 49/0 29 114769} Ke8 {-0.26 47/0 29 107238}
52. f3 {+0.26 50/0 67 210016} Kd7 {-0.27 55/0 37 138573}
53. g4 {+0.26 73/0 27 107958} hxg4 {-0.27 66/0 59 204145}
54. f4 {+0.15 101/0 29 141682} Kd8 {-0.15 60/0 26 118046}
55. Kf2 {+0.15 63/0 26 106422} Ke7 {-0.15 82/0 26 112012}
56. Kg2 {+0.15 71/0 29 104261} Kd7 {-0.15 64/0 27 113014}
57. Kg3 {+0.15 85/0 28 111060} Ke7 {-0.15 59/0 26 87490}
58. Kxg4 {+0.15 92/0 45 154041} Kd7 {-0.15 71/0 38 126265}
59. Kf3 {+0.15 91/0 40 135128} Ke8 {-0.15 78/0 27 79179}
60. Ke2 {+0.15 96/0 32 93464} Kd7 {-0.15 83/0 26 77364}
61. Kf2 {+0.15 95/0 53 172330} Ke7 {0.00 102/0 27 105366}
62. Ke3 {0.00 99/0 24 80258} Kd7 {0.00 114/0 30 84026}
63. Kd3 {0.00 126/0 24 84204}
Kd8 {0.00 118/0 27 75738, White's connection stalls} 0-1
"""

uci_command = pgn_to_uci_command(starting_fen, pgn)
print(uci_command)
