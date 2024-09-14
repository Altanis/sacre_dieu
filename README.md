# Sacre Dieu

<a href=".">
    <img src="./assets/icon.png" alt="sacredieu logo" height="200" width="200" />
</a>

A UCI compliant chess engine that may or may not be French.
<!-- todo improving lmr, reduction in pv node -->

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
- [x] Move Ordering Heuristics
    - [x] MVV-LVA
    - [x] Hash Move Ordering
    - [x] History Heuristic with Gravity
    - [x] Killer Moves
    - [x] SEE (strong)
- [x] HCE
    - [x] Material Evaluation
    - [x] Piece Square Tables
- [x] Quiescence Search (Captures + Promotions)
- [x] Principal Variation Search
- [x] Transposition Table
    - [x] Data Structure
    - [x] Cutoffs 
- [x] Reverse Futility Pruning
- [x] Null Move Pruning
- [x] Late Move Reduction
- [x] Late Move Pruning
- [x] Check Extensions
- [x] Aspiration Windows
- [x] Soft TM
- [x] Improving Heuristic
- [ ] Quiescent SEE Pruning
- [ ] PVS SEE Pruning
- [ ] Continuation History
- [ ] Capture History
- [ ] History Pruning
- [ ] Singular Extensions
- [ ] Multicut
- [ ] Double/Triple/Negative Extensions
- [ ] Cutnode
- [ ] Static Eval Correction History
- [ ] Futility Pruning
- [ ] Quiescent Futility Pruning
- [ ] Internal Iterative Reduction
- [ ] NNUE
- [ ] Threading