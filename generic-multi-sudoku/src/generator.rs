use crate::board::{Board, BOX_SIZE};
use crate::config::PuzzleConfig;
use crate::generic_board::GenericMultiBoard;
use crate::solver::{count_solutions, solve};

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn range(&mut self, max: usize) -> usize {
        (self.next() % max as u64) as usize
    }

    fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.range(i + 1);
            slice.swap(i, j);
        }
    }
}

fn get_seed() -> u64 {
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).unwrap_or(());
    u64::from_le_bytes(buf)
}

/// Fill a 3x3 box on a board with random digits 1-9.
fn fill_box(board: &mut Board, box_idx: usize, rng: &mut Rng) {
    let br = box_idx / 3;
    let bc = box_idx % 3;
    let start_r = br * BOX_SIZE;
    let start_c = bc * BOX_SIZE;
    let mut nums: Vec<u8> = (1..=9).collect();
    rng.shuffle(&mut nums);
    let mut idx = 0;
    for r in start_r..start_r + BOX_SIZE {
        for c in start_c..start_c + BOX_SIZE {
            board.set(r, c, nums[idx]);
            idx += 1;
        }
    }
}

/// Determine a generation order for boards using BFS from board 0.
/// Returns the order and, for each board (except the root), which
/// previously-generated board it overlaps with and the shared meta-cells.
fn generation_order(
    config: &PuzzleConfig,
) -> Vec<(usize, Option<(usize, Vec<(usize, usize)>)>)> {
    let n = config.boards.len();
    let overlap_info = config.compute_overlaps();

    // Build adjacency: board -> [(neighbor, shared_meta_cells)]
    let mut adj: Vec<Vec<(usize, Vec<(usize, usize)>)>> = vec![vec![]; n];
    let overlaps = overlap_info.overlap_meta_cells();
    for &((mr, mc), ref owners) in &overlaps {
        for i in 0..owners.len() {
            for j in (i + 1)..owners.len() {
                let a = owners[i];
                let b = owners[j];
                // Add to adjacency (accumulate shared cells)
                if let Some(entry) = adj[a].iter_mut().find(|(nb, _)| *nb == b) {
                    entry.1.push((mr, mc));
                } else {
                    adj[a].push((b, vec![(mr, mc)]));
                }
                if let Some(entry) = adj[b].iter_mut().find(|(nb, _)| *nb == a) {
                    entry.1.push((mr, mc));
                } else {
                    adj[b].push((a, vec![(mr, mc)]));
                }
            }
        }
    }

    // BFS from board 0
    let mut visited = vec![false; n];
    let mut order = Vec::new();
    let mut queue = std::collections::VecDeque::new();

    visited[0] = true;
    queue.push_back(0);
    order.push((0, None));

    while let Some(current) = queue.pop_front() {
        for (neighbor, shared) in &adj[current] {
            if !visited[*neighbor] {
                visited[*neighbor] = true;
                order.push((*neighbor, Some((current, shared.clone()))));
                queue.push_back(*neighbor);
            }
        }
    }

    // Handle disconnected components (boards with no overlaps)
    for i in 0..n {
        if !visited[i] {
            visited[i] = true;
            order.push((i, None));
        }
    }

    order
}

/// Copy overlap data from a solved source board to a destination board.
/// shared_meta_cells: meta-cells shared between src and dst.
fn copy_overlaps(
    config: &PuzzleConfig,
    src_idx: usize,
    dst_idx: usize,
    src_board: &Board,
    dst_board: &mut Board,
    shared_meta_cells: &[(usize, usize)],
) {
    for &(mr, mc) in shared_meta_cells {
        // Determine which box this meta-cell is within each board
        let src_box_r = mr - config.boards[src_idx].meta_row;
        let src_box_c = mc - config.boards[src_idx].meta_col;
        let src_box = src_box_r * 3 + src_box_c;

        let dst_box_r = mr - config.boards[dst_idx].meta_row;
        let dst_box_c = mc - config.boards[dst_idx].meta_col;
        let dst_box = dst_box_r * 3 + dst_box_c;

        src_board.copy_box_to(src_box, dst_board, dst_box);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Cells to remove per board
    fn cells_to_remove(self) -> usize {
        match self {
            Difficulty::Easy => 32,
            Difficulty::Medium => 40,
            Difficulty::Hard => 48,
        }
    }
}

/// Generate a complete multi-sudoku puzzle for the given config.
pub fn generate(config: &PuzzleConfig, difficulty: Difficulty) -> (GenericMultiBoard, GenericMultiBoard) {
    let mut rng = Rng::new(get_seed());
    let order = generation_order(config);

    let mut solution = GenericMultiBoard::new(config);

    // Phase 1: Generate solutions in topological order
    for &(board_idx, ref parent_info) in &order {
        let mut board = Board::new();

        if let Some((parent_idx, shared)) = parent_info {
            // Pre-fill shared boxes from parent
            copy_overlaps(
                config,
                *parent_idx,
                board_idx,
                &solution.boards[*parent_idx],
                &mut board,
                shared,
            );
            // Fill non-overlapping diagonal boxes
            let occupied: Vec<usize> = shared
                .iter()
                .map(|&(mr, mc)| {
                    let br = mr - config.boards[board_idx].meta_row;
                    let bc = mc - config.boards[board_idx].meta_col;
                    br * 3 + bc
                })
                .collect();
            for box_idx in [0, 4, 8] {
                if !occupied.contains(&box_idx) {
                    fill_box(&mut board, box_idx, &mut rng);
                }
            }
        } else {
            // Root board or disconnected: fill diagonal boxes freely
            fill_box(&mut board, 0, &mut rng);
            fill_box(&mut board, 4, &mut rng);
            fill_box(&mut board, 8, &mut rng);
        }

        solve(&mut board);
        solution.boards[board_idx] = board;
    }

    // Sync overlap regions (ensure consistency: later boards' overlap cells
    // match earlier boards)
    let overlap_info = config.compute_overlaps();
    for &((mr, mc), ref owners) in &overlap_info.overlap_meta_cells() {
        if owners.len() == 2 {
            let a = owners[0];
            let b = owners[1];
            let a_box_r = mr - config.boards[a].meta_row;
            let a_box_c = mc - config.boards[a].meta_col;
            let a_box = a_box_r * 3 + a_box_c;
            let b_box_r = mr - config.boards[b].meta_row;
            let b_box_c = mc - config.boards[b].meta_col;
            let b_box = b_box_r * 3 + b_box_c;
            // Copy from the board that was generated first (lower order)
            let a_order = order.iter().position(|&(idx, _)| idx == a).unwrap();
            let b_order = order.iter().position(|&(idx, _)| idx == b).unwrap();
            if a_order < b_order {
                let src = solution.boards[a].clone();
                src.copy_box_to(a_box, &mut solution.boards[b], b_box);
            } else {
                let src = solution.boards[b].clone();
                src.copy_box_to(b_box, &mut solution.boards[a], a_box);
            }
        }
    }

    // Phase 2: Remove cells to create the puzzle
    let mut puzzle = solution.clone();
    let to_remove = difficulty.cells_to_remove();

    for b in 0..config.boards.len() {
        let mut positions: Vec<(usize, usize)> = Vec::with_capacity(81);
        for r in 0..9 {
            for c in 0..9 {
                positions.push((r, c));
            }
        }
        rng.shuffle(&mut positions);

        let mut removed = 0;
        for (r, c) in positions {
            if removed >= to_remove {
                break;
            }
            if puzzle.boards[b].is_empty(r, c) {
                continue;
            }

            let val = puzzle.boards[b].get(r, c);
            puzzle.boards[b].set(r, c, 0);

            // If overlap, also clear the partner board's corresponding cell
            let (or, oc) = config.board_offset(b);
            let gr = or + r;
            let gc = oc + c;
            let owners = overlap_info.owners_of_cell(gr, gc);
            let mut neighbor_state: Vec<(usize, usize, usize, u8)> = Vec::new();

            for &nb in &owners {
                if nb != b {
                    let (nlr, nlc) = config.grid_to_local(nb, gr, gc);
                    let nv = puzzle.boards[nb].get(nlr, nlc);
                    neighbor_state.push((nb, nlr, nlc, nv));
                    puzzle.boards[nb].set(nlr, nlc, 0);
                }
            }

            // Check uniqueness for this board and any affected neighbors
            let mut unique = true;
            let mut test = puzzle.boards[b].clone();
            if count_solutions(&mut test, 2) != 1 {
                unique = false;
            }
            if unique {
                for &(nb, _, _, _) in &neighbor_state {
                    let mut test_n = puzzle.boards[nb].clone();
                    if count_solutions(&mut test_n, 2) != 1 {
                        unique = false;
                        break;
                    }
                }
            }

            if unique {
                removed += 1;
            } else {
                // Restore
                puzzle.boards[b].set(r, c, val);
                for &(nb, nlr, nlc, nv) in &neighbor_state {
                    puzzle.boards[nb].set(nlr, nlc, nv);
                }
            }
        }
    }

    (puzzle, solution)
}
