use serde::{Deserialize, Serialize};

pub const SIZE: usize = 9;
pub const BOX_SIZE: usize = 3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub cells: [[u8; SIZE]; SIZE],
}

impl Board {
    pub fn new() -> Self {
        Board {
            cells: [[0; SIZE]; SIZE],
        }
    }

    pub fn get(&self, row: usize, col: usize) -> u8 {
        self.cells[row][col]
    }

    pub fn set(&mut self, row: usize, col: usize, val: u8) {
        self.cells[row][col] = val;
    }

    pub fn is_empty(&self, row: usize, col: usize) -> bool {
        self.cells[row][col] == 0
    }

    pub fn is_valid_placement(&self, row: usize, col: usize, num: u8) -> bool {
        // Check row
        for c in 0..SIZE {
            if self.cells[row][c] == num {
                return false;
            }
        }
        // Check column
        for r in 0..SIZE {
            if self.cells[r][col] == num {
                return false;
            }
        }
        // Check 3x3 box
        let box_row = (row / BOX_SIZE) * BOX_SIZE;
        let box_col = (col / BOX_SIZE) * BOX_SIZE;
        for r in box_row..box_row + BOX_SIZE {
            for c in box_col..box_col + BOX_SIZE {
                if self.cells[r][c] == num {
                    return false;
                }
            }
        }
        true
    }

    pub fn candidates(&self, row: usize, col: usize) -> Vec<u8> {
        if !self.is_empty(row, col) {
            return vec![];
        }
        (1..=9)
            .filter(|&n| self.is_valid_placement(row, col, n))
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        for r in 0..SIZE {
            for c in 0..SIZE {
                if self.cells[r][c] == 0 {
                    return false;
                }
            }
        }
        true
    }

    pub fn to_flat_array(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(81);
        for r in 0..SIZE {
            for c in 0..SIZE {
                result.push(self.cells[r][c]);
            }
        }
        result
    }

    pub fn from_flat_array(data: &[u8]) -> Option<Self> {
        if data.len() != 81 {
            return None;
        }
        let mut board = Board::new();
        for (i, &val) in data.iter().enumerate() {
            board.cells[i / SIZE][i % SIZE] = val;
        }
        Some(board)
    }
}
