# WebGames — Game Backlog

Games already implemented are listed in `docs/src/roadmap/existing-games.md`.
The items below are candidates for future implementation.

---

## Number / Math Puzzles

- [ ] **Nonogram (Picross)** — Reveal a pixel image by filling grid cells using row/column number clues.
- [ ] **Kakuro** — Crossword-style grid where cells in each run must sum to a clue value using unique digits 1–9.
- [ ] **KenKen** — Sudoku-like grid where caged regions must reach a target value via a given operator (+, −, ×, ÷).
- [ ] **Takuzu / Binairo** — Fill a binary grid (0s and 1s) satisfying row/column balance and uniqueness constraints.

## Tile Sliding / Merging

- [ ] **Threes!** — Predecessor to 2048; merge 1s, 2s, and multiples of 3 on a 4×4 grid.
- [ ] **Drop7** — Drop numbered discs into columns; clear a disc when its number equals the count of discs in its row or column.
- [ ] **Klotski / Sliding Block Puzzle** — Slide rectangular blocks to navigate a target piece to the exit.

## Color / Pattern Sorting

- [ ] **Flood Fill** — Repeatedly flood the board from a corner with a chosen color to capture the whole grid in as few moves as possible.
- [ ] **Flow Free** — Connect matching colored dots with pipes such that every cell is covered.
- [ ] **Hue Rotation / Kami** — Fold a patterned board of colored triangles to a single color in a limited number of steps.

## Word / Symbol Logic

- [ ] **Wordle** — Guess a 5-letter word with color-coded feedback (correct / present / absent) in six attempts.
- [ ] **Minesweeper** — Uncover all safe cells on a grid using numeric neighbor clues to avoid hidden mines.

## Hex Puzzles

- [ ] **HexPuzzle** — Sliding/launching puzzle on a hexagonal grid.
  - The board is a hexagonal grid of hexagon tiles.
  - Each hex has a fixed movement direction — one of the six cardinal hex directions (one per side).
  - **Goal:** remove every hex from the board.
  - **Gameplay loop:**
    1. The player selects a hex.
    2. The hex slides in its assigned direction as far as possible within the grid.
    3. If it reaches the board edge and falls off, it is removed.  
       If it is blocked before the edge, it snaps back to its original position.
    4. Repeat until the board is empty.
