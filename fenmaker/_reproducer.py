import chess
import chess.pgn
import io
import re

starting_fen = "rn1qk2r/ppp1bppp/4pn2/6Bb/2BP4/2N2N1P/PPP2PP1/R2QK2R w KQkq - 0 1"
pgn = """
[Event "?"]
[Site "?"]
[Date "2024.09.15"]
[Round "1"]
[White "SacreDieu-dev"]
[Black "SacreDieu-base"]
[Result "1-0"]
[FEN "rn1qk2r/ppp1bppp/4pn2/6Bb/2BP4/2N2N1P/PPP2PP1/R2QK2R w KQkq - 0 1"]
[GameDuration "00:00:22"]
[GameEndTime "2024-09-15T03:45:03.517 UTC"]
[GameStartTime "2024-09-15T03:44:40.652 UTC"]
[PlyCount "57"]
[SetUp "1"]
[Termination "stalled connection"]
[TimeControl "3.3+0.03"]

1. Bxf6 {+0.23 12/0 234 520610} Bxf6 {-0.34 13/0 188 425453}
2. d5 {+0.31 13/0 188 433512} exd5 {-0.12 14/0 367 830599}
3. Qxd5 {+0.12 13/0 365 774438} Bxc3+ {-0.17 15/0 220 488502}
4. bxc3 {+0.19 16/0 197 474795} Qxd5 {-0.10 16/0 223 538377}
5. Bxd5 {+0.21 17/0 195 484309} Nc6 {-0.16 16/0 146 361097}
6. Bxc6+ {+0.21 16/0 176 438537} bxc6 {-0.12 18/0 179 459265}
7. O-O-O {+0.33 18/0 178 446206} Bxf3 {-0.88 18/0 373 987879}
8. Rhe1+ {+0.34 17/0 121 333439} Kf8 {-0.18 17/0 192 513511}
9. gxf3 {+0.23 18/0 188 513658} Re8 {-0.21 17/0 185 506815}
10. Rg1 {+0.22 17/0 139 380135} g6 {-0.21 15/0 120 313250}
11. Rd7 {+0.29 16/0 217 586471} Re7 {-0.33 15/0 132 356970}
12. Rgd1 {+0.34 13/0 91 243952} h5 {-0.33 15/0 100 234267}
13. R1d4 {+0.32 14/0 92 214511} a5 {-0.33 16/0 92 263100}
14. Kd2 {+0.28 15/0 93 261900} h4 {-0.47 17/0 185 526728}
15. Rd8+ {+0.52 17/0 84 228977} Re8 {-0.52 18/0 88 257709}
16. R4d7 {+0.60 19/0 93 256583} Rxd8 {-0.59 20/0 71 210601}
17. Rxd8+ {+0.60 20/0 90 251074} Kg7 {-0.60 22/0 69 207354}
18. Rxh8 {+0.60 20/0 120 217511} Kxh8 {-0.07 23/0 227 683607}
19. Ke3 {-0.11 23/0 142 450792} Kg7 {+0.11 22/0 107 334805}
20. Kf4 {-0.05 22/0 94 276570} Kf6 {+0.09 23/0 57 175492}
21. a4 {-0.13 22/0 56 167919} g5+ {+0.11 26/0 50 161002}
22. Kg4 {-0.25 29/0 133 444248} Kg6 {+0.25 31/0 57 193591}
23. f4 {-0.25 35/0 55 191320} f5+ {+0.25 31/0 45 154409}
24. Kf3 {-0.25 37/0 51 173441} gxf4 {+0.25 37/0 48 169030}
25. Kxf4 {-0.25 37/0 164 481978} Kf6 {+0.25 38/0 55 183120}
26. c4 {-0.23 35/0 131 363257} c5 {+0.25 38/0 47 151059}
27. c3 {-0.25 37/0 70 196223} c6 {+0.25 39/0 50 176870}
28. f3 {-0.25 39/0 37 121738} Ke6 {+0.25 42/0 44 145269}
29. Ke3 {-0.25 40/0 34 110949}
"""

move_info = []

def pgn_to_uci_command(starting_fen: str, pgn: str) -> str:
    _, per_move_pgn = pgn.strip().split("\n\n")
    for move in per_move_pgn:
        _, real_move_str = move.split(". ")
        movedata = re.findall(r'[^{} ]+|\{[^}]+\}', real_move_str.strip())

        if len(movedata) == 4:
            move1_nodes = movedata[1].split(" ")[3]
            move2_nodes = movedata[3].split(" ")[3]

            move_info.append({ "move": movedata[0], "nodes": int(move1_nodes[:-1]) })
            move_info.append({ "move": movedata[2], "nodes": int(move2_nodes[:-1]) })
        elif len(movedata) == 2:
            move1_nodes = movedata[1].split(" ")[3]
            move_info.append({ "move": movedata[0], "nodes": int(move1_nodes[:-1]) })
        else:
            print("disaster")

    "e"

uci_command = pgn_to_uci_command(starting_fen, pgn)
print(move_info)
print(uci_command)
