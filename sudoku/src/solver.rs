use crate::board::{Board, SIZE};

/// Find the empty cell with the fewest candidates (MRV heuristic)
fn find_best_empty(board: &Board) -> Option<(usize, usize, Vec<u8>)> {
    let mut best: Option<(usize, usize, Vec<u8>)> = None;
    for r in 0..SIZE {
        for c in 0..SIZE {
            if board.is_empty(r, c) {
                let cands = board.candidates(r, c);
                if cands.is_empty() {
                    return Some((r, c, cands)); // dead end
                }
                if best.is_none() || cands.len() < best.as_ref().unwrap().2.len() {
                    best = Some((r, c, cands));
                }
            }
        }
    }
    best
}

/// Solve the board in place using backtracking with constraint propagation.
/// Returns true if a solution was found.
pub fn solve(board: &mut Board) -> bool {
    let cell = find_best_empty(board);
    let (row, col, candidates) = match cell {
        None => return true, // no empty cells = solved
        Some(c) => c,
    };

    for num in candidates {
        board.set(row, col, num);
        if solve(board) {
            return true;
        }
        board.set(row, col, 0);
    }
    false
}

/// Count solutions up to a limit. Used to verify unique solution puzzles.
pub fn count_solutions(board: &mut Board, limit: usize) -> usize {
    let cell = find_best_empty(board);
    let (row, col, candidates) = match cell {
        None => return 1,
        Some(c) => c,
    };

    let mut count = 0;
    for num in candidates {
        board.set(row, col, num);
        count += count_solutions(board, limit - count);
        board.set(row, col, 0);
        if count >= limit {
            break;
        }
    }
    count
}

/// Verify that a completed board is a valid sudoku solution.
pub fn verify(board: &Board) -> bool {
    // Check all cells are filled
    if !board.is_complete() {
        return false;
    }
    // Check each row
    for r in 0..SIZE {
        let mut seen = [false; 10];
        for c in 0..SIZE {
            let v = board.get(r, c) as usize;
            if v == 0 || v > 9 || seen[v] {
                return false;
            }
            seen[v] = true;
        }
    }
    // Check each column
    for c in 0..SIZE {
        let mut seen = [false; 10];
        for r in 0..SIZE {
            let v = board.get(r, c) as usize;
            if v == 0 || v > 9 || seen[v] {
                return false;
            }
            seen[v] = true;
        }
    }
    // Check each 3x3 box
    for box_r in 0..3 {
        for box_c in 0..3 {
            let mut seen = [false; 10];
            for r in 0..3 {
                for c in 0..3 {
                    let v = board.get(box_r * 3 + r, box_c * 3 + c) as usize;
                    if v == 0 || v > 9 || seen[v] {
                        return false;
                    }
                    seen[v] = true;
                }
            }
        }
    }
    true
}

/// Verify partial board: check that no constraint is violated (duplicates)
/// but allow empty cells.
pub fn verify_partial(board: &Board) -> bool {
    // Check each row
    for r in 0..SIZE {
        let mut seen = [false; 10];
        for c in 0..SIZE {
            let v = board.get(r, c) as usize;
            if v == 0 {
                continue;
            }
            if v > 9 || seen[v] {
                return false;
            }
            seen[v] = true;
        }
    }
    // Check each column
    for c in 0..SIZE {
        let mut seen = [false; 10];
        for r in 0..SIZE {
            let v = board.get(r, c) as usize;
            if v == 0 {
                continue;
            }
            if v > 9 || seen[v] {
                return false;
            }
            seen[v] = true;
        }
    }
    // Check each 3x3 box
    for box_r in 0..3 {
        for box_c in 0..3 {
            let mut seen = [false; 10];
            for r in 0..3 {
                for c in 0..3 {
                    let v = board.get(box_r * 3 + r, box_c * 3 + c) as usize;
                    if v == 0 {
                        continue;
                    }
                    if v > 9 || seen[v] {
                        return false;
                    }
                    seen[v] = true;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_empty_board() {
        let mut board = Board::new();
        assert!(solve(&mut board));
        assert!(verify(&board));
    }

    #[test]
    fn test_verify_invalid() {
        let mut board = Board::new();
        board.set(0, 0, 1);
        board.set(0, 1, 1); // duplicate in row
        assert!(!verify_partial(&board));
    }
}
