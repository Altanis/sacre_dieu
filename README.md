# Sacre Dieu

<a href=".">
    <img src="./assets/icon.png" alt="sacredieu logo" height="200" width="200" />
</a>

A UCI compliant chess engine that may or may not be French.
<!-- todo: pvs tt cutoffs, tt cutoffs in qsearch, killers, see pruning, fix mate scores -->

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
    - [x] History Heuristic with Gravity
    - [x] Killer Moves
    - [ ] SEE (strong)
- [x] HCE
    - [x] Material Evaluation
    - [x] Piece Square Tables
- [x] Quiescence Search (Captures + Promotions)
- [ ] Principal Variation Search
- [x] Transposition Table
    - [x] Data Structure
    - [x] Cutoffs 
- [ ] Passed Pawn Detection
- [ ] Null Move Pruning
- [ ] Reverse + Futility Pruning
- [ ] Delta Pruning
- [ ] Late Move Reduction
- [ ] Check Extensions
- [ ] Singular Extensions
- [ ] Razoring
- [ ] Aspiration Windows
- [ ] NNUE
- [ ] Threading