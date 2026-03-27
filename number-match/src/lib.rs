mod grid;

use wasm_bindgen::prelude::*;

use grid::{Grid, Rng, Cell, COLS, MAX_ROWS, SCORE_MATCH, SCORE_CLEAR_BONUS, SCORE_ADD_ROW_PENALTY};

#[wasm_bindgen]
pub struct NumberMatchGame {
    grid: Grid,
    rng: Rng,
    score: i32,
    matches_made: u32,
    rows_added: u32,
    history: Vec<(Grid, i32, u32, u32)>, // (grid, score, matches, rows_added)
    game_over: bool,
    game_won: bool,
}

#[wasm_bindgen]
impl NumberMatchGame {
    /// Create a new game with the given number of initial rows.
    #[wasm_bindgen(constructor)]
    pub fn new(initial_rows: usize) -> NumberMatchGame {
        let mut rng = Rng::from_entropy();
        let rows = if initial_rows < 2 { 3 } else { initial_rows };
        let grid = Grid::generate(rows, &mut rng);
        NumberMatchGame {
            grid,
            rng,
            score: 0,
            matches_made: 0,
            rows_added: 0,
            history: Vec::new(),
            game_over: false,
            game_won: false,
        }
    }

    /// Get grid as flat JSON array (0=empty, 1-9=digit).
    pub fn get_grid_json(&self) -> String {
        serde_json::to_string(&self.grid.to_flat()).unwrap()
    }

    /// Number of columns (always 9).
    pub fn cols(&self) -> usize {
        COLS
    }

    /// Current number of rows.
    pub fn rows(&self) -> usize {
        self.grid.rows
    }

    /// Current score.
    pub fn score(&self) -> i32 {
        self.score
    }

    /// Number of matches made.
    pub fn matches_made(&self) -> u32 {
        self.matches_made
    }

    /// Number of rows added.
    pub fn rows_added(&self) -> u32 {
        self.rows_added
    }

    /// Remaining non-empty cells.
    pub fn remaining(&self) -> usize {
        self.grid.remaining_count()
    }

    /// Whether the game is won.
    pub fn is_won(&self) -> bool {
        self.game_won
    }

    /// Whether the game is over (won or lost).
    pub fn is_over(&self) -> bool {
        self.game_over
    }

    /// Check if a match between two flat indices is valid.
    pub fn is_valid_match(&self, a: usize, b: usize) -> bool {
        self.grid.is_valid_match(a, b)
    }

    /// Execute a match. Returns true if valid and executed.
    pub fn make_match(&mut self, a: usize, b: usize) -> bool {
        if self.game_over {
            return false;
        }
        if !self.grid.is_valid_match(a, b) {
            return false;
        }

        // Save state for undo
        self.history.push((
            self.grid.clone(),
            self.score,
            self.matches_made,
            self.rows_added,
        ));

        self.grid.set(a, Cell::Empty);
        self.grid.set(b, Cell::Empty);
        self.score += SCORE_MATCH;
        self.matches_made += 1;

        // Trim trailing empty rows
        self.grid.trim_trailing_empty_rows();

        // Check win
        if self.grid.is_cleared() {
            self.score += SCORE_CLEAR_BONUS;
            self.game_won = true;
            self.game_over = true;
        }

        true
    }

    /// Add a new row of random digits. Returns false if at max height.
    pub fn add_row(&mut self) -> bool {
        if self.game_over {
            return false;
        }
        if self.grid.rows >= MAX_ROWS {
            self.game_over = true;
            return false;
        }

        self.history.push((
            self.grid.clone(),
            self.score,
            self.matches_made,
            self.rows_added,
        ));

        self.grid.add_row(&mut self.rng);
        self.score += SCORE_ADD_ROW_PENALTY;
        self.rows_added += 1;
        true
    }

    /// Undo the last action. Returns true if successful.
    pub fn undo(&mut self) -> bool {
        if let Some((grid, score, matches, rows)) = self.history.pop() {
            self.grid = grid;
            self.score = score;
            self.matches_made = matches;
            self.rows_added = rows;
            self.game_over = false;
            self.game_won = false;
            true
        } else {
            false
        }
    }

    /// Check if any valid match exists.
    pub fn has_moves(&self) -> bool {
        self.grid.has_valid_match()
    }

    /// Get a hint: returns JSON [a, b] or "null".
    pub fn get_hint(&self) -> String {
        match self.grid.find_hint() {
            Some((a, b)) => serde_json::to_string(&[a, b]).unwrap(),
            None => "null".to_string(),
        }
    }

    /// How many undos are available.
    pub fn undo_count(&self) -> usize {
        self.history.len()
    }

    /// Get the value at a specific cell (0=empty).
    pub fn cell_value(&self, idx: usize) -> u8 {
        match self.grid.get(idx) {
            Cell::Digit(v) => v,
            Cell::Empty => 0,
        }
    }
}
