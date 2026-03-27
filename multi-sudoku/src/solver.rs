use crate::board::{Board, SIZE};

fn find_best_empty(board: &Board) -> Option<(usize, usize, Vec<u8>)> {
    let mut best: Option<(usize, usize, Vec<u8>)> = None;
    for r in 0..SIZE {
        for c in 0..SIZE {
            if board.is_empty(r, c) {
                let cands = board.candidates(r, c);
                if cands.is_empty() {
                    return Some((r, c, cands));
                }
                if best.is_none() || cands.len() < best.as_ref().unwrap().2.len() {
                    best = Some((r, c, cands));
                }
            }
        }
    }
    best
}

pub fn solve(board: &mut Board) -> bool {
    let cell = find_best_empty(board);
    let (row, col, candidates) = match cell {
        None => return true,
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

pub fn verify(board: &Board) -> bool {
    if !board.is_complete() {
        return false;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_empty_board() {
        let mut board = Board::new();
        assert!(solve(&mut board));
        assert!(verify(&board));
    }
}
