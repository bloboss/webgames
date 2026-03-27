use crate::game::{Bottle, GameState, BOTTLE_CAPACITY};
use crate::solver;

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

/// Difficulty controls number of colors.
#[derive(Clone, Copy, Debug)]
pub enum Difficulty {
    Easy,   // 4 colors, 6 bottles
    Medium, // 7 colors, 9 bottles
    Hard,   // 10 colors, 12 bottles
}

impl Difficulty {
    pub fn num_colors(self) -> usize {
        match self {
            Difficulty::Easy => 4,
            Difficulty::Medium => 7,
            Difficulty::Hard => 10,
        }
    }

    pub fn num_empty(self) -> usize {
        2
    }
}

/// Generate a solvable water sort puzzle.
pub fn generate(difficulty: Difficulty) -> GameState {
    let mut rng = Rng::new(get_seed());
    let num_colors = difficulty.num_colors();
    let num_empty = difficulty.num_empty();
    let num_bottles = num_colors + num_empty;

    loop {
        // Create the pool of color units: 4 of each color
        let mut pool: Vec<u8> = Vec::with_capacity(num_colors * BOTTLE_CAPACITY);
        for color in 1..=num_colors as u8 {
            for _ in 0..BOTTLE_CAPACITY {
                pool.push(color);
            }
        }
        rng.shuffle(&mut pool);

        // Distribute into bottles
        let mut bottles: Vec<Bottle> = Vec::with_capacity(num_bottles);
        for i in 0..num_colors {
            let start = i * BOTTLE_CAPACITY;
            let end = start + BOTTLE_CAPACITY;
            bottles.push(Bottle::from_units(pool[start..end].to_vec()));
        }
        // Add empty bottles
        for _ in 0..num_empty {
            bottles.push(Bottle::new());
        }

        // Reject trivially solved or nearly-solved puzzles
        let state = GameState::new(bottles, num_colors);
        if state.is_solved() {
            continue;
        }

        // Verify solvability
        if solver::is_solvable(&state) {
            return state;
        }
        // If not solvable, reshuffle and try again
    }
}

pub fn difficulty_from_str(s: &str) -> Difficulty {
    match s {
        "easy" => Difficulty::Easy,
        "hard" => Difficulty::Hard,
        _ => Difficulty::Medium,
    }
}
