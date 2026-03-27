use serde::{Deserialize, Serialize};

pub const BOTTLE_CAPACITY: usize = 4;

/// A bottle is a stack of colored units (bottom to top).
/// Color 0 = empty slot.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bottle {
    /// Units stored bottom-first: units[0] is bottom, units[len-1] is top.
    pub units: Vec<u8>,
}

impl Bottle {
    pub fn new() -> Self {
        Bottle { units: Vec::new() }
    }

    pub fn from_units(units: Vec<u8>) -> Self {
        Bottle { units }
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.units.len() >= BOTTLE_CAPACITY
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn free_space(&self) -> usize {
        BOTTLE_CAPACITY - self.units.len()
    }

    pub fn top_color(&self) -> Option<u8> {
        self.units.last().copied()
    }

    /// Count how many contiguous same-color units are on top.
    pub fn top_count(&self) -> usize {
        if self.is_empty() {
            return 0;
        }
        let top = *self.units.last().unwrap();
        let mut count = 0;
        for &u in self.units.iter().rev() {
            if u == top {
                count += 1;
            } else {
                break;
            }
        }
        count
    }

    /// Check if this bottle is "solved": all same color or empty.
    pub fn is_pure(&self) -> bool {
        if self.is_empty() {
            return true;
        }
        let first = self.units[0];
        self.units.iter().all(|&u| u == first)
    }

    /// Check if bottle is complete: full and pure.
    pub fn is_complete(&self) -> bool {
        self.is_full() && self.is_pure()
    }

    /// Push a color unit onto the top.
    pub fn push(&mut self, color: u8) {
        self.units.push(color);
    }

    /// Pop the top unit.
    pub fn pop(&mut self) -> Option<u8> {
        self.units.pop()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameState {
    pub bottles: Vec<Bottle>,
    pub num_colors: usize,
    pub moves: u32,
}

impl GameState {
    pub fn new(bottles: Vec<Bottle>, num_colors: usize) -> Self {
        GameState {
            bottles,
            num_colors,
            moves: 0,
        }
    }

    /// Check if the puzzle is solved.
    pub fn is_solved(&self) -> bool {
        self.bottles.iter().all(|b| b.is_empty() || b.is_complete())
    }

    /// Check if a pour from `src` to `dst` is valid.
    pub fn can_pour(&self, src: usize, dst: usize) -> bool {
        if src == dst || src >= self.bottles.len() || dst >= self.bottles.len() {
            return false;
        }
        let s = &self.bottles[src];
        let d = &self.bottles[dst];

        if s.is_empty() || d.is_full() {
            return false;
        }

        // Don't pour a complete/pure-full bottle
        if s.is_complete() {
            return false;
        }

        // Destination must be empty or top color must match
        if d.is_empty() {
            // Don't pour into empty if source is already pure (pointless move)
            // unless there are other empty bottles (optional optimization)
            return true;
        }

        s.top_color() == d.top_color()
    }

    /// Execute a pour from `src` to `dst`. Returns number of units poured.
    pub fn pour(&mut self, src: usize, dst: usize) -> usize {
        if !self.can_pour(src, dst) {
            return 0;
        }

        let src_top = self.bottles[src].top_color().unwrap();
        let src_count = self.bottles[src].top_count();
        let dst_space = self.bottles[dst].free_space();
        let to_pour = src_count.min(dst_space);

        for _ in 0..to_pour {
            self.bottles[src].pop();
        }
        for _ in 0..to_pour {
            self.bottles[dst].push(src_top);
        }

        self.moves += 1;
        to_pour
    }

    /// Get a canonical representation for state deduplication in solver.
    /// Sort bottles to treat permutations as identical.
    pub fn canonical(&self) -> Vec<Vec<u8>> {
        let mut sorted: Vec<Vec<u8>> = self.bottles.iter().map(|b| b.units.clone()).collect();
        sorted.sort();
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottle_top_count() {
        let b = Bottle::from_units(vec![1, 2, 2, 2]);
        assert_eq!(b.top_count(), 3);
        assert_eq!(b.top_color(), Some(2));
    }

    #[test]
    fn test_pour() {
        let bottles = vec![
            Bottle::from_units(vec![1, 2]),
            Bottle::from_units(vec![2]),
        ];
        let mut game = GameState::new(bottles, 2);
        assert!(game.can_pour(0, 1));
        let poured = game.pour(0, 1);
        assert_eq!(poured, 1);
        assert_eq!(game.bottles[0].units, vec![1]);
        assert_eq!(game.bottles[1].units, vec![2, 2]);
    }

    #[test]
    fn test_solved() {
        let bottles = vec![
            Bottle::from_units(vec![1, 1, 1, 1]),
            Bottle::from_units(vec![2, 2, 2, 2]),
            Bottle::new(),
        ];
        let game = GameState::new(bottles, 2);
        assert!(game.is_solved());
    }
}
