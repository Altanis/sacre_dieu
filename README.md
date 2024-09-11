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
- [ ] Reverse Futility Pruning
- [ ] Null Move Pruning
- [ ] Late Move Reduction
- [ ] Late Move Pruning
- [ ] Futility Pruning
- [ ] Internal Iterative Reduction
- [ ] Improving Heuristic
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
- [ ] Quiescent Futility Pruning
- [ ] NNUE
- [ ] Threading