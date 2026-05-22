# Future Games

Candidates for future implementation, grouped by theme.  
See also [`TODO.md`](../../../TODO.md) in the repo root for a checkbox-style task list.

---

## Number / Math Puzzles

### Nonogram (Picross)
Reveal a pixel image by filling grid cells guided by numeric clues on each row and column.

### Kakuro
Crossword-style grid where each run of cells must sum to the clue value using unique digits 1–9.

### KenKen
Sudoku-like grid where caged regions must reach a target value via a specified arithmetic operator (+, −, ×, ÷).

### Takuzu / Binairo
Fill a binary grid (0s and 1s) so that each row and column is balanced and no two are identical.

---

## Tile Sliding / Merging

### Threes!
Predecessor to 2048 — merge 1s, 2s, and multiples of 3 on a compact 4×4 grid.

### Drop7
Drop numbered discs into columns; a disc clears when its number equals the count of discs in its row or column.

### Klotski / Sliding Block Puzzle
Slide rectangular blocks around a confined board to guide the target piece to the exit.

---

## Color / Pattern Sorting

### Flood Fill
Flood the board from a corner with a chosen color each turn; capture the entire grid in as few moves as possible.

### Flow Free
Connect each pair of matching colored dots with a pipe so that every cell on the board is covered.

### Hue Rotation / Kami
Fold a patterned board of colored triangles down to a single uniform color in a limited number of steps.

---

## Word / Symbol Logic

### Wordle
Guess a hidden 5-letter word. After each guess, tiles reveal whether each letter is correct, present, or absent.

### Minesweeper
Uncover all safe cells on a grid without hitting a mine, guided by numeric neighbor clue tiles.

---

## Hex Puzzles

### HexPuzzle

A sliding/launching puzzle played on a hexagonal grid.

**Board:** A hex grid of hexagon tiles. Each tile has a fixed, immutable movement direction — one of the six cardinal hex directions (one per side of the hexagon).

**Goal:** Remove every tile from the board.

**Gameplay loop:**
1. The player clicks/taps a hex to select it.
2. The hex slides in its assigned direction as far as it can travel within the grid boundaries.
3. **Falls off the edge** → the tile is removed from the board permanently.  
   **Blocked before the edge** → the tile snaps back to its original position.
4. Play continues until the board is empty (win) or no moves can remove any tile (stuck).

**Design notes:**
- Direction arrows (or visual indicators) on each hex communicate the movement direction to the player.
- Puzzle design is the key challenge: tile removal order matters because tiles can block each other.
- Difficulty scales with board size and the intricacy of the dependency chain required to clear all tiles.
