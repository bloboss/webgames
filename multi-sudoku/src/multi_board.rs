use crate::board::{Board, BOX_SIZE};

/// The unified grid is 21x21.
/// Board 0: grid rows 0..9,   cols 0..9
/// Board 1: grid rows 6..15,  cols 6..15
/// Board 2: grid rows 12..21, cols 12..21
///
/// Overlap 0-1: grid rows 6..9,   cols 6..9   (board 0 box 8, board 1 box 0)
/// Overlap 1-2: grid rows 12..15, cols 12..15  (board 1 box 8, board 2 box 0)
pub const GRID_SIZE: usize = 21;
pub const NUM_BOARDS: usize = 3;

/// Offset of each board's top-left corner in the unified grid
pub const BOARD_OFFSETS: [(usize, usize); NUM_BOARDS] = [(0, 0), (6, 6), (12, 12)];

#[derive(Clone, Debug)]
pub struct MultiBoard {
    pub boards: [Board; NUM_BOARDS],
}

impl MultiBoard {
    pub fn new() -> Self {
        MultiBoard {
            boards: [Board::new(), Board::new(), Board::new()],
        }
    }

    /// Check if a grid coordinate is active (belongs to at least one board)
    pub fn is_active(gr: usize, gc: usize) -> bool {
        Self::owning_boards(gr, gc).len() > 0
    }

    /// Return which board indices own this grid cell
    pub fn owning_boards(gr: usize, gc: usize) -> Vec<usize> {
        let mut owners = Vec::new();
        for (i, &(or, oc)) in BOARD_OFFSETS.iter().enumerate() {
            if gr >= or && gr < or + 9 && gc >= oc && gc < oc + 9 {
                owners.push(i);
            }
        }
        owners
    }

    /// Convert grid coords to board-local coords
    pub fn grid_to_local(board_idx: usize, gr: usize, gc: usize) -> (usize, usize) {
        let (or, oc) = BOARD_OFFSETS[board_idx];
        (gr - or, gc - oc)
    }

    /// Get value at grid position (from first owning board)
    pub fn get(&self, gr: usize, gc: usize) -> u8 {
        let owners = Self::owning_boards(gr, gc);
        if owners.is_empty() {
            return 0;
        }
        let (lr, lc) = Self::grid_to_local(owners[0], gr, gc);
        self.boards[owners[0]].get(lr, lc)
    }

    /// Set value at grid position, updating all owning boards
    pub fn set(&mut self, gr: usize, gc: usize, val: u8) {
        let owners = Self::owning_boards(gr, gc);
        for &b in &owners {
            let (lr, lc) = Self::grid_to_local(b, gr, gc);
            self.boards[b].set(lr, lc, val);
        }
    }

    pub fn is_empty(&self, gr: usize, gc: usize) -> bool {
        self.get(gr, gc) == 0
    }

    /// Check if the grid cell is in an overlap zone
    pub fn is_overlap(gr: usize, gc: usize) -> bool {
        Self::owning_boards(gr, gc).len() > 1
    }

    /// Which board "box" index does this grid cell's 3x3 belong to, for a given board
    pub fn box_index_for_board(board_idx: usize, gr: usize, gc: usize) -> usize {
        let (lr, lc) = Self::grid_to_local(board_idx, gr, gc);
        (lr / BOX_SIZE) * 3 + (lc / BOX_SIZE)
    }

    /// Check if all three boards are complete and valid
    pub fn is_solved(&self) -> bool {
        use crate::solver::verify;
        self.boards.iter().all(|b| verify(b))
    }

    /// Count empty cells across the unified grid (no double-counting overlaps)
    pub fn empty_count(&self) -> usize {
        let mut count = 0;
        for gr in 0..GRID_SIZE {
            for gc in 0..GRID_SIZE {
                if Self::is_active(gr, gc) && self.is_empty(gr, gc) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Count errors vs solution (no double-counting)
    pub fn error_count(&self, solution: &MultiBoard) -> usize {
        let mut count = 0;
        for gr in 0..GRID_SIZE {
            for gc in 0..GRID_SIZE {
                if Self::is_active(gr, gc) {
                    let v = self.get(gr, gc);
                    if v != 0 && v != solution.get(gr, gc) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Sync overlap zones from board with lower index to higher index
    pub fn sync_overlaps(&mut self) {
        // Overlap 0-1: board 0 box 8 -> board 1 box 0
        let (left, right) = self.boards.split_at_mut(1);
        left[0].copy_box_to(8, &mut right[0], 0);
        // Overlap 1-2: board 1 box 8 -> board 2 box 0
        let (left, right) = self.boards[1..].split_at_mut(1);
        left[0].copy_box_to(8, &mut right[0], 0);
    }
}
