use serde::{Deserialize, Serialize};

/// Axial hex coordinate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
}

impl Hex {
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn step(self, dir: Direction) -> Hex {
        let (dq, dr) = dir.delta();
        Hex::new(self.q + dq, self.r + dr)
    }
}

/// Six axial directions. Indexes are stable for serialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Direction {
    E = 0,
    NE = 1,
    NW = 2,
    W = 3,
    SW = 4,
    SE = 5,
}

impl Direction {
    pub const ALL: [Direction; 6] = [
        Direction::E,
        Direction::NE,
        Direction::NW,
        Direction::W,
        Direction::SW,
        Direction::SE,
    ];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::E => (1, 0),
            Direction::NE => (1, -1),
            Direction::NW => (0, -1),
            Direction::W => (-1, 0),
            Direction::SW => (-1, 1),
            Direction::SE => (0, 1),
        }
    }

    /// Angle in degrees from the hex center toward the neighbour in this
    /// direction, using screen coordinates (0° = right, 90° = down).
    pub fn angle_deg(self) -> f64 {
        match self {
            Direction::E => 0.0,
            Direction::SE => 60.0,
            Direction::SW => 120.0,
            Direction::W => 180.0,
            Direction::NW => 240.0,
            Direction::NE => 300.0,
        }
    }

    pub fn from_index(i: usize) -> Option<Direction> {
        Self::ALL.get(i).copied()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Tile(Direction),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    pub radius: i32,
    /// List of (coordinate, cell) for every position on the board.
    pub cells: Vec<(Hex, Cell)>,
    pub moves: u32,
    pub removed: u32,
    pub total: u32,
}

impl GameState {
    /// Build an empty hex-shaped board with the given radius.
    pub fn empty(radius: i32) -> Self {
        let mut cells = Vec::new();
        for q in -radius..=radius {
            let r_min = (-radius).max(-q - radius);
            let r_max = radius.min(-q + radius);
            for r in r_min..=r_max {
                cells.push((Hex::new(q, r), Cell::Empty));
            }
        }
        Self {
            radius,
            cells,
            moves: 0,
            removed: 0,
            total: 0,
        }
    }

    /// Generate a solvable puzzle by reverse construction.
    ///
    /// We "place tiles back" onto the board: at each step we pick an empty
    /// cell and a direction such that, from that cell, the path off the
    /// board in that direction is clear. The puzzle is then solvable by
    /// removing tiles in the inverse order of placement.
    pub fn generate(radius: i32, target_tiles: u32, rng: &mut Rng) -> (Self, Vec<Hex>) {
        let mut state = Self::empty(radius);
        let mut order: Vec<Hex> = Vec::new();
        let mut attempts = 0u32;
        let max_attempts = target_tiles * 200;

        while (order.len() as u32) < target_tiles && attempts < max_attempts {
            attempts += 1;
            let idx = rng.range(state.cells.len());
            if !matches!(state.cells[idx].1, Cell::Empty) {
                continue;
            }
            let hex = state.cells[idx].0;
            let dir_offset = rng.range(6);
            for k in 0..6 {
                let dir = Direction::from_index((dir_offset + k) % 6).unwrap();
                if state.path_clear(hex, dir) {
                    state.set(hex, Cell::Tile(dir));
                    order.push(hex);
                    break;
                }
            }
        }

        state.total = order.len() as u32;
        (state, order)
    }

    pub fn on_board(&self, hex: Hex) -> bool {
        let r = self.radius;
        hex.q.abs() <= r && hex.r.abs() <= r && (hex.q + hex.r).abs() <= r
    }

    fn index_of(&self, hex: Hex) -> Option<usize> {
        self.cells.iter().position(|(h, _)| *h == hex)
    }

    pub fn get(&self, hex: Hex) -> Option<Cell> {
        self.index_of(hex).map(|i| self.cells[i].1)
    }

    pub fn set(&mut self, hex: Hex, cell: Cell) {
        if let Some(i) = self.index_of(hex) {
            self.cells[i].1 = cell;
        }
    }

    /// True if the line from `hex` in direction `dir` (exclusive of the
    /// starting cell) contains only empty cells, all the way off the board.
    pub fn path_clear(&self, hex: Hex, dir: Direction) -> bool {
        let mut cur = hex.step(dir);
        while self.on_board(cur) {
            if !matches!(self.get(cur), Some(Cell::Empty)) {
                return false;
            }
            cur = cur.step(dir);
        }
        true
    }

    /// Count empty on-board cells along the line from `hex` in direction
    /// `dir` (exclusive of the starting cell), stopping at the first
    /// non-empty cell or at the edge of the board.
    pub fn empty_steps(&self, hex: Hex, dir: Direction) -> i32 {
        let mut cur = hex.step(dir);
        let mut n = 0;
        while self.on_board(cur) {
            if !matches!(self.get(cur), Some(Cell::Empty)) {
                return n;
            }
            cur = cur.step(dir);
            n += 1;
        }
        n
    }

    /// Attempt to play the tile at `hex`. Returns `Move::Removed` if the
    /// tile slides off, `Move::Blocked` if blocked (no state change), and
    /// `Move::Empty` if the clicked cell has no tile.
    pub fn play(&mut self, hex: Hex) -> Move {
        let cell = match self.get(hex) {
            Some(c) => c,
            None => return Move::Empty,
        };
        let dir = match cell {
            Cell::Tile(d) => d,
            Cell::Empty => return Move::Empty,
        };
        self.moves += 1;
        if self.path_clear(hex, dir) {
            self.set(hex, Cell::Empty);
            self.removed += 1;
            Move::Removed
        } else {
            Move::Blocked
        }
    }

    pub fn is_won(&self) -> bool {
        self.cells.iter().all(|(_, c)| matches!(c, Cell::Empty))
    }

    /// True if no tile can currently slide off the board.
    pub fn is_stuck(&self) -> bool {
        for (hex, cell) in &self.cells {
            if let Cell::Tile(dir) = cell {
                if self.path_clear(*hex, *dir) {
                    return false;
                }
            }
        }
        // Stuck only matters if there are still tiles left.
        self.cells.iter().any(|(_, c)| matches!(c, Cell::Tile(_)))
    }

    pub fn tile_count(&self) -> u32 {
        self.cells.iter().filter(|(_, c)| matches!(c, Cell::Tile(_))).count() as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Removed,
    Blocked,
    Empty,
}

/// Simple xorshift64 PRNG.
#[derive(Clone, Debug)]
pub struct Rng {
    pub state: u64,
}

impl Rng {
    pub fn from_entropy() -> Self {
        let mut buf = [0u8; 8];
        let _ = getrandom::fill(&mut buf);
        let state = u64::from_le_bytes(buf);
        Rng {
            state: if state == 0 { 0x9E3779B97F4A7C15 } else { state },
        }
    }

    pub fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn range(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.next() % max as u64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_radius_2_has_19_cells() {
        let s = GameState::empty(2);
        assert_eq!(s.cells.len(), 19);
    }

    #[test]
    fn tile_with_clear_path_falls_off() {
        let mut s = GameState::empty(2);
        s.set(Hex::new(0, 0), Cell::Tile(Direction::E));
        assert_eq!(s.play(Hex::new(0, 0)), Move::Removed);
        assert!(matches!(s.get(Hex::new(0, 0)), Some(Cell::Empty)));
        assert_eq!(s.removed, 1);
    }

    #[test]
    fn tile_blocked_by_other_tile_snaps_back() {
        let mut s = GameState::empty(2);
        s.set(Hex::new(0, 0), Cell::Tile(Direction::E));
        s.set(Hex::new(2, 0), Cell::Tile(Direction::W));
        let before = s.get(Hex::new(0, 0));
        assert_eq!(s.play(Hex::new(0, 0)), Move::Blocked);
        assert_eq!(s.get(Hex::new(0, 0)), before);
    }

    #[test]
    fn generate_produces_solvable_puzzle() {
        let mut rng = Rng { state: 42 };
        let (mut s, order) = GameState::generate(3, 10, &mut rng);
        assert!(s.total >= 1);

        // Solve in reverse placement order, which is guaranteed to work.
        for hex in order.iter().rev() {
            assert_eq!(s.play(*hex), Move::Removed, "should remove tile at {:?}", hex);
        }
        assert_eq!(s.tile_count(), 0);
    }
}
