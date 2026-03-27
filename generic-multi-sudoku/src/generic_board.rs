use crate::board::Board;
use crate::config::{OverlapInfo, PuzzleConfig};
use crate::solver::verify;

/// Dynamic multi-board: manages N overlapping 9x9 boards
/// based on an arbitrary PuzzleConfig.
#[derive(Clone, Debug)]
pub struct GenericMultiBoard {
    pub config: PuzzleConfig,
    pub boards: Vec<Board>,
    pub overlap_info: OverlapInfo,
}

impl GenericMultiBoard {
    pub fn new(config: &PuzzleConfig) -> Self {
        let overlap_info = config.compute_overlaps();
        let boards = vec![Board::new(); config.boards.len()];
        GenericMultiBoard {
            config: config.clone(),
            boards,
            overlap_info,
        }
    }

    pub fn grid_rows(&self) -> usize {
        self.overlap_info.grid_rows
    }

    pub fn grid_cols(&self) -> usize {
        self.overlap_info.grid_cols
    }

    pub fn is_active(&self, gr: usize, gc: usize) -> bool {
        self.overlap_info.is_active(gr, gc)
    }

    pub fn is_overlap(&self, gr: usize, gc: usize) -> bool {
        self.overlap_info.is_overlap(gr, gc)
    }

    pub fn owning_boards(&self, gr: usize, gc: usize) -> Vec<usize> {
        self.overlap_info.owners_of_cell(gr, gc)
    }

    /// Get value from the first owning board.
    pub fn get(&self, gr: usize, gc: usize) -> u8 {
        let owners = self.owning_boards(gr, gc);
        if owners.is_empty() {
            return 0;
        }
        let (lr, lc) = self.config.grid_to_local(owners[0], gr, gc);
        self.boards[owners[0]].get(lr, lc)
    }

    /// Set value on all owning boards.
    pub fn set(&mut self, gr: usize, gc: usize, val: u8) {
        let owners = self.owning_boards(gr, gc);
        for &b in &owners {
            let (lr, lc) = self.config.grid_to_local(b, gr, gc);
            self.boards[b].set(lr, lc, val);
        }
    }

    pub fn is_empty(&self, gr: usize, gc: usize) -> bool {
        self.get(gr, gc) == 0
    }

    /// All boards are complete and valid.
    pub fn is_solved(&self) -> bool {
        self.boards.iter().all(|b| verify(b))
    }

    /// Count empty cells in the unified grid (no double-counting).
    pub fn empty_count(&self) -> usize {
        let mut count = 0;
        for gr in 0..self.grid_rows() {
            for gc in 0..self.grid_cols() {
                if self.is_active(gr, gc) && self.is_empty(gr, gc) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Count errors vs solution (no double-counting).
    pub fn error_count(&self, solution: &Self) -> usize {
        let mut count = 0;
        for gr in 0..self.grid_rows() {
            for gc in 0..self.grid_cols() {
                if self.is_active(gr, gc) {
                    let v = self.get(gr, gc);
                    if v != 0 && v != solution.get(gr, gc) {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}
