use serde::{Deserialize, Serialize};

pub const GRID_SIZE: usize = 8;
pub const TOTAL_CELLS: usize = GRID_SIZE * GRID_SIZE;
pub const MAX_TILES: usize = 64;
pub const SPAWN_INTERVAL_MS: u32 = 3000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Value(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub cells: Vec<Cell>,
    pub score: u64,
    pub merges: u32,
    pub highest: u32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            cells: vec![Cell::Empty; TOTAL_CELLS],
            score: 0,
            merges: 0,
            highest: 0,
        }
    }

    pub fn empty_indices(&self) -> Vec<usize> {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(i, c)| if *c == Cell::Empty { Some(i) } else { None })
            .collect()
    }

    pub fn tile_count(&self) -> usize {
        self.cells.iter().filter(|c| **c != Cell::Empty).count()
    }

    pub fn spawn_tile(&mut self, rng: &mut Rng) -> bool {
        let empty = self.empty_indices();
        if empty.is_empty() {
            return false;
        }
        let idx = empty[rng.range(empty.len())];
        self.cells[idx] = Cell::Value(1);
        true
    }

    pub fn can_merge(&self, a: usize, b: usize) -> bool {
        if a == b {
            return false;
        }
        match (self.cells[a], self.cells[b]) {
            (Cell::Value(va), Cell::Value(vb)) => va == vb,
            _ => false,
        }
    }

    pub fn merge(&mut self, a: usize, b: usize) -> Option<u32> {
        if !self.can_merge(a, b) {
            return None;
        }
        if let Cell::Value(v) = self.cells[a] {
            let new_val = v * 2;
            self.cells[b] = Cell::Value(new_val);
            self.cells[a] = Cell::Empty;
            self.score += new_val as u64;
            self.merges += 1;
            if new_val > self.highest {
                self.highest = new_val;
            }
            Some(new_val)
        } else {
            None
        }
    }
}

/// Simple xorshift64 PRNG.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn from_entropy() -> Self {
        let mut buf = [0u8; 8];
        getrandom::fill(&mut buf).unwrap_or(());
        let state = u64::from_le_bytes(buf);
        Rng {
            state: if state == 0 { 1 } else { state },
        }
    }

    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn range(&mut self, max: usize) -> usize {
        (self.next() % max as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_tile() {
        let mut state = GameState::new();
        let mut rng = Rng { state: 12345 };
        assert!(state.spawn_tile(&mut rng));
        assert_eq!(state.tile_count(), 1);
    }

    #[test]
    fn test_merge_same_value() {
        let mut state = GameState::new();
        state.cells[0] = Cell::Value(2);
        state.cells[1] = Cell::Value(2);
        let result = state.merge(0, 1);
        assert_eq!(result, Some(4));
        assert_eq!(state.cells[0], Cell::Empty);
        assert_eq!(state.cells[1], Cell::Value(4));
        assert_eq!(state.score, 4);
        assert_eq!(state.merges, 1);
    }

    #[test]
    fn test_merge_different_value() {
        let mut state = GameState::new();
        state.cells[0] = Cell::Value(2);
        state.cells[1] = Cell::Value(4);
        assert!(!state.can_merge(0, 1));
        assert_eq!(state.merge(0, 1), None);
    }

    #[test]
    fn test_board_full() {
        let mut state = GameState::new();
        for i in 0..TOTAL_CELLS {
            state.cells[i] = Cell::Value(1);
        }
        let mut rng = Rng { state: 12345 };
        assert!(!state.spawn_tile(&mut rng));
    }
}
