use serde::{Deserialize, Serialize};

pub const GRID_SIZE: usize = 5;
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

    /// Check if dragging `src` onto `dst` is a valid merge.
    pub fn can_merge(&self, src: usize, dst: usize) -> bool {
        if src == dst || src >= TOTAL_CELLS || dst >= TOTAL_CELLS {
            return false;
        }
        match (self.cells[src], self.cells[dst]) {
            (Cell::Value(a), Cell::Value(b)) => a == b,
            _ => false,
        }
    }

    /// Execute a merge: double the value at dst, clear src.
    /// Returns the new merged value.
    pub fn merge(&mut self, src: usize, dst: usize) -> Option<u32> {
        if !self.can_merge(src, dst) {
            return None;
        }
        let val = self.cells[src].value().unwrap();
        let merged = val * 2;
        self.cells[dst] = Cell::Value(merged);
        self.cells[src] = Cell::Empty;
        self.score += merged as u64;
        self.merges += 1;
        if merged > self.highest {
            self.highest = merged;
        }
        Some(merged)
    }

    /// Place a value at an index.
    pub fn place(&mut self, idx: usize, val: u32) {
        self.cells[idx] = Cell::Value(val);
        if val > self.highest {
            self.highest = val;
        }
    }

    /// Check if any merge is possible on the board.
    pub fn has_valid_merge(&self) -> bool {
        for i in 0..TOTAL_CELLS {
            if let Cell::Value(v) = self.cells[i] {
                // Check right neighbor
                let (r, c) = Self::to_rc(i);
                if c + 1 < GRID_SIZE {
                    if self.cells[Self::to_idx(r, c + 1)] == Cell::Value(v) {
                        return true;
                    }
                }
                // Check bottom neighbor
                if r + 1 < GRID_SIZE {
                    if self.cells[Self::to_idx(r + 1, c)] == Cell::Value(v) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check game over: grid full and no merges possible.
    pub fn is_game_over(&self) -> bool {
        self.empty_count() == 0 && !self.has_valid_merge()
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
    fn test_merge() {
        let mut board = Board::new();
        board.place(0, 2);
        board.place(1, 2);
        assert!(board.can_merge(0, 1));
        let result = board.merge(0, 1);
        assert_eq!(result, Some(4));
        assert_eq!(board.cells[0], Cell::Empty);
        assert_eq!(board.cells[1], Cell::Value(4));
        assert_eq!(board.score, 4);
    }

    #[test]
    fn test_no_merge_different_values() {
        let mut board = Board::new();
        board.place(0, 2);
        board.place(1, 4);
        assert!(!board.can_merge(0, 1));
    }

    #[test]
    fn test_no_merge_empty() {
        let board = Board::new();
        assert!(!board.can_merge(0, 1));
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
        // Make two adjacent cells the same
        board.cells[0] = Cell::Value(2);
        board.cells[1] = Cell::Value(2);
        assert!(!board.is_game_over());
    }

    #[test]
    fn test_has_valid_merge() {
        let mut board = Board::new();
        board.place(0, 4);
        board.place(1, 4);
        assert!(board.has_valid_merge());
    }

    #[test]
    fn test_highest_tracking() {
        let mut board = Board::new();
        board.place(0, 2);
        board.place(1, 2);
        board.merge(0, 1);
        assert_eq!(board.highest, 4);
    }
}
