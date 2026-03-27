mod game;
mod generator;
mod solver;

use wasm_bindgen::prelude::*;

use game::{Bottle, GameState};

/// The WASM-exposed game handle.
#[wasm_bindgen]
pub struct WaterSortGame {
    initial: GameState,
    state: GameState,
    history: Vec<GameState>,
}

#[wasm_bindgen]
impl WaterSortGame {
    /// Create a new game. difficulty: "easy", "medium", or "hard".
    #[wasm_bindgen(constructor)]
    pub fn new(difficulty: &str) -> WaterSortGame {
        let diff = generator::difficulty_from_str(difficulty);
        let state = generator::generate(diff);
        let initial = state.clone();
        WaterSortGame {
            initial,
            state,
            history: Vec::new(),
        }
    }

    /// Number of bottles.
    pub fn num_bottles(&self) -> usize {
        self.state.bottles.len()
    }

    /// Number of colors used.
    pub fn num_colors(&self) -> usize {
        self.state.num_colors
    }

    /// Get bottle contents as JSON: [[1,2,3,4],[2,1],...].
    pub fn get_bottles_json(&self) -> String {
        let data: Vec<Vec<u8>> = self.state.bottles.iter().map(|b| b.units.clone()).collect();
        serde_json::to_string(&data).unwrap()
    }

    /// Get move count.
    pub fn move_count(&self) -> u32 {
        self.state.moves
    }

    /// Check if a pour is valid.
    pub fn can_pour(&self, src: usize, dst: usize) -> bool {
        self.state.can_pour(src, dst)
    }

    /// Execute a pour. Returns number of units poured (0 if invalid).
    pub fn pour(&mut self, src: usize, dst: usize) -> usize {
        if !self.state.can_pour(src, dst) {
            return 0;
        }
        self.history.push(self.state.clone());
        self.state.pour(src, dst)
    }

    /// Undo the last move. Returns true if successful.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.state = prev;
            true
        } else {
            false
        }
    }

    /// Check if the puzzle is solved.
    pub fn is_solved(&self) -> bool {
        self.state.is_solved()
    }

    /// Reset to initial state.
    pub fn reset(&mut self) {
        self.state = self.initial.clone();
        self.history.clear();
    }

    /// Get a hint: returns JSON [src, dst] or null if no solution.
    pub fn get_hint(&self) -> String {
        match solver::solve(&self.state) {
            Some(moves) if !moves.is_empty() => {
                let m = &moves[0];
                serde_json::to_string(&[m.src, m.dst]).unwrap()
            }
            _ => "null".to_string(),
        }
    }

    /// Solve from current state. Returns JSON array of [src, dst] pairs, or null.
    pub fn solve(&self) -> String {
        match solver::solve(&self.state) {
            Some(moves) => {
                let pairs: Vec<[usize; 2]> = moves.iter().map(|m| [m.src, m.dst]).collect();
                serde_json::to_string(&pairs).unwrap()
            }
            None => "null".to_string(),
        }
    }

    /// How many undos are available.
    pub fn undo_count(&self) -> usize {
        self.history.len()
    }
}
