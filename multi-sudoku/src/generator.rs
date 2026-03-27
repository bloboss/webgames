use crate::board::{Board, BOX_SIZE};
use crate::multi_board::MultiBoard;
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

fn fill_diagonal_boxes(board: &mut Board, rng: &mut Rng, skip_box_0: bool) {
    for box_idx in [0, 4, 8] {
        if skip_box_0 && box_idx == 0 {
            continue; // box 0 is pre-filled from overlap
        }
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
}

fn generate_solved_first(rng: &mut Rng) -> Board {
    let mut board = Board::new();
    fill_diagonal_boxes(&mut board, rng, false);
    solve(&mut board);
    board
}

fn generate_solved_chained(prev: &Board, rng: &mut Rng) -> Board {
    let mut board = Board::new();
    // Copy box 8 of prev into box 0 of this board
    prev.copy_box_to(8, &mut board, 0);
    // Fill diagonal boxes 4 and 8 (box 0 is already set)
    fill_diagonal_boxes(&mut board, rng, true);
    solve(&mut board);
    board
}

#[derive(Clone, Copy, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Cells to remove per board (out of 81)
    fn cells_to_remove(self) -> usize {
        match self {
            Difficulty::Easy => 32,
            Difficulty::Medium => 40,
            Difficulty::Hard => 48,
        }
    }
}

pub fn difficulty_from_str(s: &str) -> Difficulty {
    match s {
        "easy" => Difficulty::Easy,
        "hard" => Difficulty::Hard,
        _ => Difficulty::Medium,
    }
}

/// Generate multi-sudoku puzzle. Returns (puzzle, solution).
pub fn generate(difficulty: Difficulty) -> (MultiBoard, MultiBoard) {
    let mut rng = Rng::new(get_seed());

    // Generate three chained solutions
    let board0 = generate_solved_first(&mut rng);
    let board1 = generate_solved_chained(&board0, &mut rng);
    let board2 = generate_solved_chained(&board1, &mut rng);

    let solution = MultiBoard {
        boards: [board0, board1, board2],
    };
    let mut puzzle = solution.clone();

    let to_remove = difficulty.cells_to_remove();

    // Remove cells from each board independently, but shared cells affect both
    for b in 0..3 {
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

            // If this cell is in an overlap zone, also clear the neighbor board
            let mut neighbor_val = None;
            if b < 2 && r >= 6 && c >= 6 {
                // This is in the overlap with board b+1 (box 8 of b = box 0 of b+1)
                let nr = r - 6;
                let nc = c - 6;
                neighbor_val = Some((b + 1, nr, nc, puzzle.boards[b + 1].get(nr, nc)));
                puzzle.boards[b + 1].set(nr, nc, 0);
            } else if b > 0 && r < 3 && c < 3 {
                // This is in the overlap with board b-1 (box 0 of b = box 8 of b-1)
                let nr = r + 6;
                let nc = c + 6;
                neighbor_val = Some((b - 1, nr, nc, puzzle.boards[b - 1].get(nr, nc)));
                puzzle.boards[b - 1].set(nr, nc, 0);
            }

            // Check uniqueness for this board
            let mut test = puzzle.boards[b].clone();
            let unique = count_solutions(&mut test, 2) == 1;

            // Also check neighbor if overlap
            let neighbor_unique = if let Some((nb, _, _, _)) = neighbor_val {
                let mut test_n = puzzle.boards[nb].clone();
                count_solutions(&mut test_n, 2) == 1
            } else {
                true
            };

            if unique && neighbor_unique {
                removed += 1;
            } else {
                // Restore
                puzzle.boards[b].set(r, c, val);
                if let Some((nb, nr, nc, nv)) = neighbor_val {
                    puzzle.boards[nb].set(nr, nc, nv);
                }
            }
        }
    }

    (puzzle, solution)
}
