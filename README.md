# Sacre Dieu

A UCI compliant chess engine which may or may not be French.

## Releases
- `v1_basic`: negamax + a/b pruning + iterative deepening, only
- `v2_mvv_lva`: v1 + mvv-lva + super naive see
- `v3_psqt`: v2 + psqt tables
- `v4_qsearch`: v3 + quiescence search
- `v5_qsearch_ext`: v4 + noisy promotions
- `v6_tuned_psqt` v5 + tuned psqt values
- `v7_fail_soft`: v6, but search/qsearch are fail soft now
- `v8_draw_score`: v7, but fixing how draw scores work

todo: fix draw checks, TT table search, TT table qsearch, TT table move ordering, fix mate scores

## Features
- [x] FEN Parser
- [x] Bitboards
- [x] Magic Bitboards
- [x] UCI Compliancy
- [x] Negamax with A/B Pruning
- [x] Iterative Deepening
- [x] Draw Detection
    - [x] Threefold Repetition
    - [x] 50-Move Rule
- [x] Mate Distance Pruning
- [ ] Move Ordering Heuristics
    - [x] MVV-LVA
    - [x] SEE (naive)
    - [ ] SEE (strong)
- [ ] HCE
    - [x] Material Evaluation
    - [x] Piece Square Tables
    - [ ] Tuned HCE
- [x] Quiescence Search (Captures + Promotions)
- [ ] Transposition Table
- [ ] PVS
- [ ] Killer Moves
- [ ] Passed Pawn Detection
- [ ] Reverse Futility Pruning
- [ ] Delta Pruning
- [ ] Late Move Reduction
- [ ] Check Extensions
- [ ] Hash Move Ordering
- [ ] History Heuristic
- [ ] Razoring
- [ ] Aspiration Tables
- [ ] NNUE
- [ ] Threading