# Sacre Dieu

<a href=".">
    <img src="./assets/icon.png" alt="sacredieu logo" height="200" width="200" />
</a>

A UCI compliant chess engine that may or may not be French.

## Releases
- `v1_basic`: negamax + a/b pruning + iterative deepening, only
- `v2_mvv_lva`: v1 + mvv-lva + super naive see
- `v3_psqt`: v2 + psqt tables
- `v4_qsearch`: v3 + quiescence search
- `v5_qsearch_ext`: v4 + noisy promotions
- `v6_tuned_psqt` v5 + tuned psqt values
- `v7_fail_soft`: v6, but search/qsearch are fail soft now
- `v8_draw_score`: v7, but fixing how draw scores work
- `v9_tt_move_ordering`: v8 + tt move ordering
- `v10_tt_cutoff`: v9 + tt cutoffs
- `v11_fixed_tt_move`: v10 + fixed tt hash move probing

<!-- todo: history (gravity), killers, SEE, PVS, TT table search, TT table qsearch, TT table move ordering, fix mate scores -->

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
    - [x] Hash Move Ordering
    - [ ] History Heuristic with Gravity
    - [ ] Killer Moves
    - [ ] SEE (strong)
- [ ] HCE
    - [x] Material Evaluation
    - [x] Piece Square Tables
    - [ ] Tuned HCE
- [x] Quiescence Search (Captures + Promotions)
- [ ] Principal Variation Search
- [ ] Transposition Table
    - [x] Data Structure
    - [ ] Cutoffs 
- [ ] Passed Pawn Detection
- [ ] Futility Pruning
- [ ] Delta Pruning
- [ ] Late Move Reduction
- [ ] Check Extensions
- [ ] Singular Extensions
- [ ] Razoring
- [ ] Aspiration Tables
- [ ] NNUE
- [ ] Threading