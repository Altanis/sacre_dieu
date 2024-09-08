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
starting_fen = "r2qk1nr/pp2ppbp/2n3p1/8/2P1pPb1/2P2NP1/PP2Q2P/RNB1KB1R w KQkq - 0 9"
pgn = """
[Event "Fast-Chess Tournament"]
[Site "?"]
[Date "2024.09.08"]
[Round "2"]
[White "BlueGarbageBall"]
[Black "Aspect"]
[Result "1-0"]
[SetUp "1"]
[FEN "r2qk1nr/pp2ppbp/2n3p1/8/2P1pPb1/2P2NP1/PP2Q2P/RNB1KB1R w KQkq - 0 9"]
[GameDuration "00:00:27"]
[GameStartTime "2024-09-08T13:31:00 -0400"]
[GameEndTime "2024-09-08T13:31:28 -0400"]
[PlyCount "52"]
[Termination "time forfeit"]
[TimeControl "8+0.08"]

1. Qxe4 {-0.06/8, 0.400s} Nf6 {+0.62/4, 0.414s} 2. Qe2 {-0.25/7, 0.419s}
Bxf3 {+0.47/4, 0.390s} 3. Qxf3 {+0.14/8, 0.367s} e5 {-0.18/5, 0.375s}
4. Qe2 {+0.08/7, 0.352s} Qc7 {+0.33/4, 0.358s} 5. fxe5 {+0.34/8, 0.339s}
Nxe5 {-0.18/5, 0.346s} 6. Bf4 {+0.38/8, 0.326s} Nd7 {-0.34/5, 0.332s}
7. Nd2 {+0.50/8, 0.314s} O-O {+0.07/4, 0.325s} 8. Bg2 {+0.32/7, 0.302s}
Rad8 {-0.19/4, 0.307s} 9. O-O-O {+0.46/6, 0.291s} Nc5 {-0.08/4, 0.299s}
10. Bxe5 {+0.39/7, 0.280s} Qxe5 {-0.34/4, 0.295s} 11. Qxe5 {+0.76/8, 0.270s}
Bxe5 {-0.39/4, 0.279s} 12. Ne4 {+0.87/8, 0.261s} b6 {-0.58/4, 0.268s}
13. Nxc5 {+1.00/8, 0.251s} bxc5 {-0.93/5, 0.267s} 14. Rhe1 {+1.05/8, 0.243s}
Rxd1+ {-1.00/5, 0.268s} 15. Rxd1 {+1.19/9, 0.237s} Rc8 {-1.05/5, 0.241s}
16. Rd5 {+1.38/8, 0.227s} Bf6 {-1.23/5, 0.230s} 17. Rd7 {+1.56/8, 0.220s}
Bg5+ {-1.45/5, 0.224s} 18. Kc2 {+1.21/9, 0.213s} Rd8 {-1.25/6, 0.213s}
19. Rd5 {+1.18/9, 0.206s} Rxd5 {-1.20/6, 0.217s} 20. cxd5 {+1.22/10, 0.200s}
Be7 {-1.33/7, 0.212s} 21. a4 {+1.60/10, 0.194s} a5 {-1.22/7, 0.205s}
22. Kd3 {+1.85/10, 0.188s} Bd6 {-1.60/7, 0.198s} 23. Kc4 {+1.99/11, 0.183s}
f5 {-1.85/7, 0.188s} 24. Kb5 {+2.22/10, 0.177s} f4 {-2.06/7, 0.184s}
25. Kc6 {+2.67/11, 0.173s} fxg3 {-1.29/8, 0.176s} 26. Kxd6 {+8.42/11, 0.168s} 1-0
"""

uci_command = pgn_to_uci_command(starting_fen, pgn)
print(uci_command)
