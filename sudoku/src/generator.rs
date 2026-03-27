use crate::board::{Board, SIZE, BOX_SIZE};
use crate::solver::{solve, count_solutions};

/// Simple pseudo-random number generator seeded from JS
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
        // xorshift64
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

/// Fill the three diagonal 3x3 boxes (they are independent of each other)
fn fill_diagonal_boxes(board: &mut Board, rng: &mut Rng) {
    for box_idx in 0..3 {
        let start = box_idx * BOX_SIZE;
        let mut nums: Vec<u8> = (1..=9).collect();
        rng.shuffle(&mut nums);
        let mut idx = 0;
        for r in start..start + BOX_SIZE {
            for c in start..start + BOX_SIZE {
                board.set(r, c, nums[idx]);
                idx += 1;
            }
        }
    }
}

/// Generate a fully solved board
fn generate_solved(rng: &mut Rng) -> Board {
    let mut board = Board::new();
    fill_diagonal_boxes(&mut board, rng);
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
    /// Number of cells to remove
    fn cells_to_remove(self) -> usize {
        match self {
            Difficulty::Easy => 36,
            Difficulty::Medium => 46,
            Difficulty::Hard => 54,
        }
    }
}

pub fn difficulty_from_str(s: &str) -> Difficulty {
    match s.to_lowercase().as_str() {
        "easy" => Difficulty::Easy,
        "medium" => Difficulty::Medium,
        "hard" => Difficulty::Hard,
        _ => Difficulty::Medium,
    }
}

/// Generate a puzzle by removing cells from a solved board,
/// ensuring a unique solution.
pub fn generate(difficulty: Difficulty) -> (Board, Board) {
    let mut rng = Rng::new(get_seed());
    let solution = generate_solved(&mut rng);
    let mut puzzle = solution.clone();

    let to_remove = difficulty.cells_to_remove();
    let mut positions: Vec<(usize, usize)> = Vec::with_capacity(81);
    for r in 0..SIZE {
        for c in 0..SIZE {
            positions.push((r, c));
        }
    }
    rng.shuffle(&mut positions);

    let mut removed = 0;
    for (r, c) in positions {
        if removed >= to_remove {
            break;
        }
        let val = puzzle.get(r, c);
        if val == 0 {
            continue;
        }
        puzzle.set(r, c, 0);

        let mut test = puzzle.clone();
        if count_solutions(&mut test, 2) == 1 {
            removed += 1;
        } else {
            puzzle.set(r, c, val); // restore if not unique
        }
    }

    (puzzle, solution)
}
