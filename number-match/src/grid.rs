use serde::{Deserialize, Serialize};

pub const COLS: usize = 9;
pub const MAX_ROWS: usize = 50;

// Scoring constants
pub const SCORE_MATCH: i32 = 10;
pub const SCORE_CLEAR_BONUS: i32 = 100;
pub const SCORE_ADD_ROW_PENALTY: i32 = -20;

/// A cell is either Empty or holds a digit 1-9.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Digit(u8),
}

impl Cell {
    pub fn is_empty(self) -> bool {
        matches!(self, Cell::Empty)
    }

    pub fn value(self) -> Option<u8> {
        match self {
            Cell::Digit(v) => Some(v),
            Cell::Empty => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid {
    /// Row-major storage. Each row has exactly COLS cells.
    pub cells: Vec<Cell>,
    pub rows: usize,
}

impl Grid {
    pub fn new(initial_digits: &[u8]) -> Self {
        let mut cells: Vec<Cell> = initial_digits.iter().map(|&d| Cell::Digit(d)).collect();
        // Pad to fill the last row if needed
        while cells.len() % COLS != 0 {
            cells.push(Cell::Empty);
        }
        let rows = cells.len() / COLS;
        Grid { cells, rows }
    }

    /// Generate an initial grid with `num_rows` rows of random digits 1-9.
    pub fn generate(num_rows: usize, rng: &mut Rng) -> Self {
        let total = num_rows * COLS;
        let mut digits = Vec::with_capacity(total);
        for _ in 0..total {
            digits.push((rng.range(9) as u8) + 1);
        }
        Grid::new(&digits)
    }

    pub fn total_cells(&self) -> usize {
        self.cells.len()
    }

    pub fn get(&self, idx: usize) -> Cell {
        if idx < self.cells.len() {
            self.cells[idx]
        } else {
            Cell::Empty
        }
    }

    pub fn set(&mut self, idx: usize, cell: Cell) {
        if idx < self.cells.len() {
            self.cells[idx] = cell;
        }
    }

    /// Convert flat index to (row, col).
    pub fn to_rc(&self, idx: usize) -> (usize, usize) {
        (idx / COLS, idx % COLS)
    }

    /// Convert (row, col) to flat index.
    pub fn to_idx(&self, row: usize, col: usize) -> usize {
        row * COLS + col
    }

    /// Check if all cells are empty (win condition).
    pub fn is_cleared(&self) -> bool {
        self.cells.iter().all(|c| c.is_empty())
    }

    /// Count remaining non-empty cells.
    pub fn remaining_count(&self) -> usize {
        self.cells.iter().filter(|c| !c.is_empty()).count()
    }

    /// Check if two cells form a valid match.
    pub fn is_valid_match(&self, a: usize, b: usize) -> bool {
        if a == b || a >= self.cells.len() || b >= self.cells.len() {
            return false;
        }

        let va = match self.cells[a].value() {
            Some(v) => v,
            None => return false,
        };
        let vb = match self.cells[b].value() {
            Some(v) => v,
            None => return false,
        };

        // Value rule: equal or sum to 10
        if va != vb && va + vb != 10 {
            return false;
        }

        // Path rule: check if cells between a and b are all empty
        self.path_clear(a, b)
    }

    /// Check if the path between two cells is clear.
    /// Tries same-row, same-column, diagonal, and linear (flattened) paths.
    fn path_clear(&self, a: usize, b: usize) -> bool {
        let (ra, ca) = self.to_rc(a);
        let (rb, cb) = self.to_rc(b);

        if ra == rb {
            // Same row: check horizontal path
            let (lo, hi) = if ca < cb { (ca, cb) } else { (cb, ca) };
            for c in (lo + 1)..hi {
                if !self.cells[self.to_idx(ra, c)].is_empty() {
                    return false;
                }
            }
            return true;
        }

        if ca == cb {
            // Same column: check vertical path
            let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
            for r in (lo + 1)..hi {
                if !self.cells[self.to_idx(r, ca)].is_empty() {
                    return false;
                }
            }
            return true;
        }

        let row_diff = if ra < rb { rb - ra } else { ra - rb };
        let col_diff = if ca < cb { cb - ca } else { ca - cb };

        if row_diff == col_diff {
            // Same diagonal: check diagonal path
            let dr: isize = if rb > ra { 1 } else { -1 };
            let dc: isize = if cb > ca { 1 } else { -1 };
            let mut r = ra as isize + dr;
            let mut c = ca as isize + dc;
            let end_r = rb as isize;
            let end_c = cb as isize;
            while r != end_r || c != end_c {
                if !self.cells[self.to_idx(r as usize, c as usize)].is_empty() {
                    return false;
                }
                r += dr;
                c += dc;
            }
            return true;
        }

        // Linear (wrapped) path: check flattened sequence
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        for i in (lo + 1)..hi {
            if !self.cells[i].is_empty() {
                return false;
            }
        }
        true
    }

    /// Add a new row of digits to the bottom of the grid.
    pub fn add_row(&mut self, rng: &mut Rng) {
        for _ in 0..COLS {
            let d = (rng.range(9) as u8) + 1;
            self.cells.push(Cell::Digit(d));
        }
        self.rows += 1;
    }

    /// Remove trailing empty rows to keep the grid compact.
    pub fn trim_trailing_empty_rows(&mut self) {
        while self.rows > 1 {
            let start = (self.rows - 1) * COLS;
            let all_empty = (start..self.cells.len()).all(|i| self.cells[i].is_empty());
            if all_empty {
                self.cells.truncate(start);
                self.rows -= 1;
            } else {
                break;
            }
        }
    }

    /// Check if any valid match exists on the board.
    pub fn has_valid_match(&self) -> bool {
        // Collect indices of non-empty cells
        let filled: Vec<usize> = self
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_empty())
            .map(|(i, _)| i)
            .collect();

        for i in 0..filled.len() {
            for j in (i + 1)..filled.len() {
                if self.is_valid_match(filled[i], filled[j]) {
                    return true;
                }
            }
        }
        false
    }

    /// Find one valid match (for hints). Returns (a, b) or None.
    pub fn find_hint(&self) -> Option<(usize, usize)> {
        let filled: Vec<usize> = self
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_empty())
            .map(|(i, _)| i)
            .collect();

        for i in 0..filled.len() {
            for j in (i + 1)..filled.len() {
                if self.is_valid_match(filled[i], filled[j]) {
                    return Some((filled[i], filled[j]));
                }
            }
        }
        None
    }

    /// Get the grid as a flat array of values (0 = empty, 1-9 = digit).
    pub fn to_flat(&self) -> Vec<u8> {
        self.cells
            .iter()
            .map(|c| match c {
                Cell::Digit(v) => *v,
                Cell::Empty => 0,
            })
            .collect()
    }
}

/// Simple PRNG.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    pub fn from_entropy() -> Self {
        let mut buf = [0u8; 8];
        getrandom::fill(&mut buf).unwrap_or(());
        Rng::new(u64::from_le_bytes(buf))
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
    fn test_same_row_match() {
        // [1, _, _, 1, 5] padded to 9 cols
        let mut cells = vec![Cell::Digit(1), Cell::Empty, Cell::Empty, Cell::Digit(1)];
        for _ in 4..9 {
            cells.push(Cell::Digit(5));
        }
        let grid = Grid {
            cells,
            rows: 1,
        };
        assert!(grid.is_valid_match(0, 3));
    }

    #[test]
    fn test_same_row_blocked() {
        let mut cells = vec![Cell::Digit(1), Cell::Digit(2), Cell::Empty, Cell::Digit(1)];
        for _ in 4..9 {
            cells.push(Cell::Digit(5));
        }
        let grid = Grid {
            cells,
            rows: 1,
        };
        assert!(!grid.is_valid_match(0, 3));
    }

    #[test]
    fn test_sum_to_10() {
        let mut cells = vec![Cell::Digit(3), Cell::Empty, Cell::Digit(7)];
        for _ in 3..9 {
            cells.push(Cell::Digit(5));
        }
        let grid = Grid {
            cells,
            rows: 1,
        };
        assert!(grid.is_valid_match(0, 2));
    }

    #[test]
    fn test_same_column_match() {
        // 2 rows, same column, cells between are empty
        let mut cells = vec![Cell::Empty; 18];
        cells[0] = Cell::Digit(4);
        cells[9] = Cell::Digit(4);
        let grid = Grid {
            cells,
            rows: 2,
        };
        assert!(grid.is_valid_match(0, 9));
    }

    #[test]
    fn test_linear_wrap_match() {
        // Two rows: row 0 ends with 3 at col 8, row 1 starts with 7 at col 0
        // All cells between (last of row 0 and first of row 1) = adjacent in flat order
        let mut cells = vec![Cell::Empty; 18];
        cells[8] = Cell::Digit(3);
        cells[9] = Cell::Digit(7); // sum = 10
        let grid = Grid {
            cells,
            rows: 2,
        };
        // They are adjacent in flat order, no cells between
        assert!(grid.is_valid_match(8, 9));
    }

    #[test]
    fn test_linear_wrap_with_empties() {
        // 3 at idx 7, empty at 8, empty at 9, 7 at idx 10
        let mut cells = vec![Cell::Empty; 18];
        cells[7] = Cell::Digit(3);
        cells[10] = Cell::Digit(7);
        let grid = Grid {
            cells,
            rows: 2,
        };
        assert!(grid.is_valid_match(7, 10));
    }

    #[test]
    fn test_is_cleared() {
        let grid = Grid {
            cells: vec![Cell::Empty; 9],
            rows: 1,
        };
        assert!(grid.is_cleared());
    }

    #[test]
    fn test_has_valid_match() {
        let mut cells = vec![Cell::Digit(5); 9];
        cells[1] = Cell::Empty;
        // cells[0]=5, cells[2]=5, but cell[1] is empty between them
        let grid = Grid {
            cells,
            rows: 1,
        };
        assert!(grid.has_valid_match());
    }

    #[test]
    fn test_no_self_match() {
        let cells = vec![Cell::Digit(5); 9];
        let grid = Grid {
            cells,
            rows: 1,
        };
        assert!(!grid.is_valid_match(0, 0));
    }

    #[test]
    fn test_diagonal_main_match() {
        // 3 rows; Digit(3) at (0,0)=idx 0 and (2,2)=idx 20; (1,1)=idx 10 is empty
        let mut cells = vec![Cell::Empty; 27];
        cells[0] = Cell::Digit(3);
        cells[20] = Cell::Digit(3);
        let grid = Grid { cells, rows: 3 };
        assert!(grid.is_valid_match(0, 20));
    }

    #[test]
    fn test_diagonal_anti_match() {
        // 3 rows; Digit(4) at (2,0)=idx 18 and Digit(6) at (0,2)=idx 2; (1,1)=idx 10 is empty
        let mut cells = vec![Cell::Empty; 27];
        cells[18] = Cell::Digit(4);
        cells[2] = Cell::Digit(6); // 4 + 6 = 10
        let grid = Grid { cells, rows: 3 };
        assert!(grid.is_valid_match(2, 18));
    }

    #[test]
    fn test_diagonal_blocked() {
        // 3 rows; Digit(5) at (0,0)=idx 0 and (2,2)=idx 20; blocker Digit(2) at (1,1)=idx 10
        let mut cells = vec![Cell::Empty; 27];
        cells[0] = Cell::Digit(5);
        cells[10] = Cell::Digit(2);
        cells[20] = Cell::Digit(5);
        let grid = Grid { cells, rows: 3 };
        assert!(!grid.is_valid_match(0, 20));
    }
}
