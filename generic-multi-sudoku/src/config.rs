use serde::{Deserialize, Serialize};

/// Where a board's top-left corner sits in meta-grid coordinates.
/// Each meta-cell = one 3x3 box, so a board occupies a 3x3 region of meta-cells.
/// Actual grid position = (meta_row * 3, meta_col * 3).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoardPlacement {
    pub meta_row: usize,
    pub meta_col: usize,
}

/// Full configuration for a generic multi-sudoku puzzle.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PuzzleConfig {
    pub name: String,
    pub boards: Vec<BoardPlacement>,
}

/// Precomputed overlap information derived from a PuzzleConfig.
#[derive(Clone, Debug)]
pub struct OverlapInfo {
    /// For each meta-cell (mr, mc), which board indices own it.
    /// Only entries with at least one owner are present.
    pub cell_owners: Vec<((usize, usize), Vec<usize>)>,
    /// Grid dimensions in actual cells.
    pub grid_rows: usize,
    pub grid_cols: usize,
    /// Maximum meta-grid dimensions.
    pub meta_rows: usize,
    pub meta_cols: usize,
}

impl OverlapInfo {
    /// Get owners for a meta-cell.
    pub fn owners_of_meta(&self, mr: usize, mc: usize) -> Vec<usize> {
        for &(key, ref owners) in &self.cell_owners {
            if key == (mr, mc) {
                return owners.clone();
            }
        }
        vec![]
    }

    /// Get owners for an actual grid cell.
    pub fn owners_of_cell(&self, gr: usize, gc: usize) -> Vec<usize> {
        let mr = gr / 3;
        let mc = gc / 3;
        self.owners_of_meta(mr, mc)
    }

    /// Check if an actual grid cell is active (belongs to at least one board).
    pub fn is_active(&self, gr: usize, gc: usize) -> bool {
        !self.owners_of_cell(gr, gc).is_empty()
    }

    /// Check if an actual grid cell is in an overlap zone (2+ boards).
    pub fn is_overlap(&self, gr: usize, gc: usize) -> bool {
        self.owners_of_cell(gr, gc).len() > 1
    }

    /// Get all meta-cells that are overlaps (owned by 2+ boards).
    pub fn overlap_meta_cells(&self) -> Vec<((usize, usize), Vec<usize>)> {
        self.cell_owners
            .iter()
            .filter(|(_, owners)| owners.len() > 1)
            .cloned()
            .collect()
    }
}

impl PuzzleConfig {
    /// Compute the board's actual grid offset.
    pub fn board_offset(&self, idx: usize) -> (usize, usize) {
        (self.boards[idx].meta_row * 3, self.boards[idx].meta_col * 3)
    }

    /// Convert grid coords to board-local coords.
    pub fn grid_to_local(&self, board_idx: usize, gr: usize, gc: usize) -> (usize, usize) {
        let (or, oc) = self.board_offset(board_idx);
        (gr - or, gc - oc)
    }

    /// Compute overlap information.
    pub fn compute_overlaps(&self) -> OverlapInfo {
        if self.boards.is_empty() {
            return OverlapInfo {
                cell_owners: vec![],
                grid_rows: 0,
                grid_cols: 0,
                meta_rows: 0,
                meta_cols: 0,
            };
        }

        let max_mr = self.boards.iter().map(|b| b.meta_row + 3).max().unwrap();
        let max_mc = self.boards.iter().map(|b| b.meta_col + 3).max().unwrap();

        let mut cell_owners = Vec::new();
        for mr in 0..max_mr {
            for mc in 0..max_mc {
                let mut owners = Vec::new();
                for (i, b) in self.boards.iter().enumerate() {
                    if mr >= b.meta_row
                        && mr < b.meta_row + 3
                        && mc >= b.meta_col
                        && mc < b.meta_col + 3
                    {
                        owners.push(i);
                    }
                }
                if !owners.is_empty() {
                    cell_owners.push(((mr, mc), owners));
                }
            }
        }

        OverlapInfo {
            cell_owners,
            grid_rows: max_mr * 3,
            grid_cols: max_mc * 3,
            meta_rows: max_mr,
            meta_cols: max_mc,
        }
    }

    /// Validate the config. Returns Err with a message if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.boards.is_empty() {
            return Err("At least one board is required.".to_string());
        }
        // Check no triple overlaps (3+ boards sharing a meta-cell)
        let info = self.compute_overlaps();
        for &(cell, ref owners) in &info.cell_owners {
            if owners.len() > 2 {
                return Err(format!(
                    "Meta-cell ({}, {}) is shared by {} boards. Maximum is 2.",
                    cell.0, cell.1, owners.len()
                ));
            }
        }
        Ok(())
    }

    /// Canonical fingerprint for keying statistics.
    /// Normalizes by sorting board placements.
    pub fn fingerprint(&self) -> String {
        let mut sorted: Vec<(usize, usize)> = self
            .boards
            .iter()
            .map(|b| (b.meta_row, b.meta_col))
            .collect();
        sorted.sort();
        // Simple FNV-1a hash of the canonical representation
        let canonical = format!("{:?}", sorted);
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in canonical.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{:016x}", hash)
    }

    /// Serialize to JSON string for saving.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

/// Built-in preset configurations.
pub fn presets() -> Vec<PuzzleConfig> {
    vec![
        PuzzleConfig {
            name: "Single".to_string(),
            boards: vec![BoardPlacement {
                meta_row: 0,
                meta_col: 0,
            }],
        },
        PuzzleConfig {
            name: "Double Diagonal".to_string(),
            boards: vec![
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 2,
                },
            ],
        },
        PuzzleConfig {
            name: "Triple Chain".to_string(),
            boards: vec![
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 2,
                },
                BoardPlacement {
                    meta_row: 4,
                    meta_col: 4,
                },
            ],
        },
        PuzzleConfig {
            name: "Samurai".to_string(),
            boards: vec![
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 4,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 2,
                },
                BoardPlacement {
                    meta_row: 4,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 4,
                    meta_col: 4,
                },
            ],
        },
        PuzzleConfig {
            name: "Cross".to_string(),
            boards: vec![
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 2,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 2,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 4,
                },
                BoardPlacement {
                    meta_row: 4,
                    meta_col: 2,
                },
            ],
        },
        PuzzleConfig {
            name: "L-Shape".to_string(),
            boards: vec![
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 2,
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_board_config() {
        let cfg = PuzzleConfig {
            name: "test".to_string(),
            boards: vec![BoardPlacement {
                meta_row: 0,
                meta_col: 0,
            }],
        };
        assert!(cfg.validate().is_ok());
        let info = cfg.compute_overlaps();
        assert_eq!(info.grid_rows, 9);
        assert_eq!(info.grid_cols, 9);
    }

    #[test]
    fn test_double_overlap() {
        let cfg = PuzzleConfig {
            name: "test".to_string(),
            boards: vec![
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 2,
                },
            ],
        };
        assert!(cfg.validate().is_ok());
        let info = cfg.compute_overlaps();
        assert_eq!(info.grid_rows, 15);
        assert_eq!(info.grid_cols, 15);
        // Meta-cell (2,2) should be owned by both boards
        let owners = info.owners_of_meta(2, 2);
        assert_eq!(owners.len(), 2);
    }

    #[test]
    fn test_triple_overlap_rejected() {
        let cfg = PuzzleConfig {
            name: "test".to_string(),
            boards: vec![
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
            ],
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_fingerprint_stable() {
        let cfg1 = PuzzleConfig {
            name: "a".to_string(),
            boards: vec![
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 2,
                },
            ],
        };
        let cfg2 = PuzzleConfig {
            name: "b".to_string(),
            boards: vec![
                BoardPlacement {
                    meta_row: 2,
                    meta_col: 2,
                },
                BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                },
            ],
        };
        assert_eq!(cfg1.fingerprint(), cfg2.fingerprint());
    }

    #[test]
    fn test_serialization() {
        let cfg = PuzzleConfig {
            name: "test".to_string(),
            boards: vec![BoardPlacement {
                meta_row: 0,
                meta_col: 0,
            }],
        };
        let json = cfg.to_json();
        let restored = PuzzleConfig::from_json(&json).unwrap();
        assert_eq!(cfg, restored);
    }
}
