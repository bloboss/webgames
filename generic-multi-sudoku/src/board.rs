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
        for c in 0..SIZE {
            if self.cells[row][c] == num {
                return false;
            }
        }
        for r in 0..SIZE {
            if self.cells[r][col] == num {
                return false;
            }
        }
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

    /// Copy a 3x3 box from this board to another.
    /// Boxes numbered 0-8 in row-major order:
    ///   0 1 2
    ///   3 4 5
    ///   6 7 8
    pub fn copy_box_to(&self, src_box: usize, dest: &mut Board, dest_box: usize) {
        let (sr, sc) = box_origin(src_box);
        let (dr, dc) = box_origin(dest_box);
        for r in 0..BOX_SIZE {
            for c in 0..BOX_SIZE {
                dest.cells[dr + r][dc + c] = self.cells[sr + r][sc + c];
            }
        }
    }

    /// Get the 3x3 box index for a cell position
    pub fn box_index(row: usize, col: usize) -> usize {
        (row / BOX_SIZE) * 3 + (col / BOX_SIZE)
    }
}

pub fn box_origin(box_idx: usize) -> (usize, usize) {
    let br = box_idx / 3;
    let bc = box_idx % 3;
    (br * BOX_SIZE, bc * BOX_SIZE)
}
