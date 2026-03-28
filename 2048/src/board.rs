use serde::{Deserialize, Serialize};

pub const GRID_SIZE: usize = 4;
pub const TOTAL_CELLS: usize = GRID_SIZE * GRID_SIZE;

/// Each cell is either empty or holds a power-of-2 value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Value(u32),
}

impl Cell {
    pub fn is_empty(self) -> bool {
        matches!(self, Cell::Empty)
    }

    pub fn value(self) -> Option<u32> {
        match self {
            Cell::Value(v) => Some(v),
            Cell::Empty => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub cells: [Cell; TOTAL_CELLS],
    pub score: u64,
    pub highest: u32,
    pub merges: u32,
}

impl Board {
    pub fn new() -> Self {
        Board {
            cells: [Cell::Empty; TOTAL_CELLS],
            score: 0,
            highest: 0,
            merges: 0,
        }
    }

    pub fn to_rc(idx: usize) -> (usize, usize) {
        (idx / GRID_SIZE, idx % GRID_SIZE)
    }

    pub fn to_idx(row: usize, col: usize) -> usize {
        row * GRID_SIZE + col
    }

    pub fn empty_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_empty()).count()
    }

    pub fn empty_indices(&self) -> Vec<usize> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_empty())
            .map(|(i, _)| i)
            .collect()
    }

    pub fn place(&mut self, idx: usize, val: u32) {
        self.cells[idx] = Cell::Value(val);
        if val > self.highest {
            self.highest = val;
        }
    }

    /// Slide and merge a single row/column of values toward index 0.
    /// Returns (new_line, score_gained, merge_count).
    fn slide_line(line: &[Cell; GRID_SIZE]) -> ([Cell; GRID_SIZE], u64, u32) {
        // Extract non-empty values
        let vals: Vec<u32> = line.iter().filter_map(|c| c.value()).collect();

        let mut score = 0u64;
        let mut merge_count = 0u32;

        // Merge adjacent equal values
        let mut merged = Vec::with_capacity(GRID_SIZE);
        let mut i = 0;
        while i < vals.len() {
            if i + 1 < vals.len() && vals[i] == vals[i + 1] {
                let new_val = vals[i] * 2;
                merged.push(new_val);
                score += new_val as u64;
                merge_count += 1;
                i += 2;
            } else {
                merged.push(vals[i]);
                i += 1;
            }
        }

        // Build result line
        let mut result = [Cell::Empty; GRID_SIZE];
        for (i, &v) in merged.iter().enumerate() {
            result[i] = Cell::Value(v);
        }

        (result, score, merge_count)
    }

    /// Execute a move in the given direction. Returns true if the board changed.
    pub fn slide(&mut self, dir: Direction) -> bool {
        let old_cells = self.cells;

        match dir {
            Direction::Left => {
                for row in 0..GRID_SIZE {
                    let mut line = [Cell::Empty; GRID_SIZE];
                    for col in 0..GRID_SIZE {
                        line[col] = self.cells[Self::to_idx(row, col)];
                    }
                    let (new_line, score, merges) = Self::slide_line(&line);
                    self.score += score;
                    self.merges += merges;
                    for col in 0..GRID_SIZE {
                        self.cells[Self::to_idx(row, col)] = new_line[col];
                    }
                }
            }
            Direction::Right => {
                for row in 0..GRID_SIZE {
                    let mut line = [Cell::Empty; GRID_SIZE];
                    for col in 0..GRID_SIZE {
                        line[GRID_SIZE - 1 - col] = self.cells[Self::to_idx(row, col)];
                    }
                    let (new_line, score, merges) = Self::slide_line(&line);
                    self.score += score;
                    self.merges += merges;
                    for col in 0..GRID_SIZE {
                        self.cells[Self::to_idx(row, col)] = new_line[GRID_SIZE - 1 - col];
                    }
                }
            }
            Direction::Up => {
                for col in 0..GRID_SIZE {
                    let mut line = [Cell::Empty; GRID_SIZE];
                    for row in 0..GRID_SIZE {
                        line[row] = self.cells[Self::to_idx(row, col)];
                    }
                    let (new_line, score, merges) = Self::slide_line(&line);
                    self.score += score;
                    self.merges += merges;
                    for row in 0..GRID_SIZE {
                        self.cells[Self::to_idx(row, col)] = new_line[row];
                    }
                }
            }
            Direction::Down => {
                for col in 0..GRID_SIZE {
                    let mut line = [Cell::Empty; GRID_SIZE];
                    for row in 0..GRID_SIZE {
                        line[GRID_SIZE - 1 - row] = self.cells[Self::to_idx(row, col)];
                    }
                    let (new_line, score, merges) = Self::slide_line(&line);
                    self.score += score;
                    self.merges += merges;
                    for row in 0..GRID_SIZE {
                        self.cells[Self::to_idx(row, col)] = new_line[GRID_SIZE - 1 - row];
                    }
                }
            }
        }

        // Update highest tile
        for cell in &self.cells {
            if let Cell::Value(v) = cell {
                if *v > self.highest {
                    self.highest = *v;
                }
            }
        }

        self.cells != old_cells
    }

    /// Check if any move is possible in any direction.
    pub fn can_move(&self) -> bool {
        // If there are empty cells, a move is always possible
        if self.empty_count() > 0 {
            return true;
        }
        // Check for adjacent equal values
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                if let Cell::Value(v) = self.cells[Self::to_idx(row, col)] {
                    // Check right
                    if col + 1 < GRID_SIZE {
                        if self.cells[Self::to_idx(row, col + 1)] == Cell::Value(v) {
                            return true;
                        }
                    }
                    // Check down
                    if row + 1 < GRID_SIZE {
                        if self.cells[Self::to_idx(row + 1, col)] == Cell::Value(v) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Check game over: no moves possible in any direction.
    pub fn is_game_over(&self) -> bool {
        !self.can_move()
    }
}

/// Simple xorshift64 PRNG.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn from_entropy() -> Self {
        let mut buf = [0u8; 8];
        getrandom::fill(&mut buf).unwrap_or(());
        let state = u64::from_le_bytes(buf);
        Rng {
            state: if state == 0 { 1 } else { state },
        }
    }

    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn range(&mut self, max: usize) -> usize {
        (self.next() % max as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slide_left_merge() {
        let mut board = Board::new();
        board.place(0, 2); // row 0, col 0
        board.place(1, 2); // row 0, col 1
        let changed = board.slide(Direction::Left);
        assert!(changed);
        assert_eq!(board.cells[0], Cell::Value(4));
        assert_eq!(board.cells[1], Cell::Empty);
        assert_eq!(board.score, 4);
    }

    #[test]
    fn test_slide_right_merge() {
        let mut board = Board::new();
        board.place(0, 2);
        board.place(1, 2);
        let changed = board.slide(Direction::Right);
        assert!(changed);
        assert_eq!(board.cells[3], Cell::Value(4));
        assert_eq!(board.cells[0], Cell::Empty);
    }

    #[test]
    fn test_slide_up_merge() {
        let mut board = Board::new();
        board.place(0, 2); // row 0, col 0
        board.place(4, 2); // row 1, col 0
        let changed = board.slide(Direction::Up);
        assert!(changed);
        assert_eq!(board.cells[0], Cell::Value(4));
        assert_eq!(board.cells[4], Cell::Empty);
    }

    #[test]
    fn test_slide_down_merge() {
        let mut board = Board::new();
        board.place(0, 2); // row 0, col 0
        board.place(4, 2); // row 1, col 0
        let changed = board.slide(Direction::Down);
        assert!(changed);
        assert_eq!(board.cells[12], Cell::Value(4)); // row 3, col 0
        assert_eq!(board.cells[0], Cell::Empty);
    }

    #[test]
    fn test_no_change_no_move() {
        let mut board = Board::new();
        board.place(0, 2);
        let changed = board.slide(Direction::Left);
        assert!(!changed); // Already at left edge, no merge
    }

    #[test]
    fn test_game_over() {
        let mut board = Board::new();
        // Fill with all different values so no merges possible
        for i in 0..TOTAL_CELLS {
            board.place(i, (i as u32 + 1) * 2);
        }
        assert!(board.is_game_over());
    }

    #[test]
    fn test_not_game_over_with_merge() {
        let mut board = Board::new();
        for i in 0..TOTAL_CELLS {
            board.place(i, (i as u32 + 1) * 2);
        }
        board.cells[0] = Cell::Value(2);
        board.cells[1] = Cell::Value(2);
        assert!(!board.is_game_over());
    }

    #[test]
    fn test_double_merge_same_row() {
        let mut board = Board::new();
        // [2, 2, 2, 2] -> slide left -> [4, 4, _, _]
        board.place(0, 2);
        board.place(1, 2);
        board.place(2, 2);
        board.place(3, 2);
        board.slide(Direction::Left);
        assert_eq!(board.cells[0], Cell::Value(4));
        assert_eq!(board.cells[1], Cell::Value(4));
        assert_eq!(board.cells[2], Cell::Empty);
        assert_eq!(board.cells[3], Cell::Empty);
    }

    #[test]
    fn test_slide_compresses() {
        let mut board = Board::new();
        // [_, 2, _, 4] -> slide left -> [2, 4, _, _]
        board.place(1, 2);
        board.place(3, 4);
        let changed = board.slide(Direction::Left);
        assert!(changed);
        assert_eq!(board.cells[0], Cell::Value(2));
        assert_eq!(board.cells[1], Cell::Value(4));
        assert_eq!(board.cells[2], Cell::Empty);
        assert_eq!(board.cells[3], Cell::Empty);
    }
}
