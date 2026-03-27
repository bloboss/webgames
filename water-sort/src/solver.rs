use std::collections::HashSet;
use std::collections::VecDeque;

use crate::game::GameState;

/// A move: pour from bottle `src` to bottle `dst`.
#[derive(Clone, Debug)]
pub struct Move {
    pub src: usize,
    pub dst: usize,
}

/// BFS solver. Returns the sequence of moves to solve, or None if unsolvable.
/// Uses canonical state representation to prune equivalent states.
pub fn solve(initial: &GameState) -> Option<Vec<Move>> {
    if initial.is_solved() {
        return Some(vec![]);
    }

    let mut visited: HashSet<Vec<Vec<u8>>> = HashSet::new();
    visited.insert(initial.canonical());

    // Queue entries: (state, moves_so_far)
    let mut queue: VecDeque<(GameState, Vec<Move>)> = VecDeque::new();
    queue.push_back((initial.clone(), vec![]));

    let max_states = 200_000;
    let mut explored = 0;

    while let Some((state, moves)) = queue.pop_front() {
        explored += 1;
        if explored > max_states {
            return None; // Too complex, give up
        }

        let n = state.bottles.len();
        for src in 0..n {
            for dst in 0..n {
                if !state.can_pour(src, dst) {
                    continue;
                }

                // Skip pouring into empty if another empty already exists after it
                // (symmetry breaking: only pour into the first empty bottle)
                if state.bottles[dst].is_empty() {
                    let first_empty = state
                        .bottles
                        .iter()
                        .position(|b| b.is_empty())
                        .unwrap();
                    if dst != first_empty {
                        continue;
                    }
                }

                let mut next = state.clone();
                next.pour(src, dst);

                if next.is_solved() {
                    let mut solution = moves.clone();
                    solution.push(Move { src, dst });
                    return Some(solution);
                }

                let canonical = next.canonical();
                if !visited.contains(&canonical) {
                    visited.insert(canonical);
                    let mut new_moves = moves.clone();
                    new_moves.push(Move { src, dst });
                    queue.push_back((next, new_moves));
                }
            }
        }
    }

    None // No solution found
}

/// Check if a puzzle is solvable (without returning the full solution path).
pub fn is_solvable(state: &GameState) -> bool {
    solve(state).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Bottle;

    #[test]
    fn test_trivial_solve() {
        let bottles = vec![
            Bottle::from_units(vec![1, 1, 1, 2]),
            Bottle::from_units(vec![2, 2, 2, 1]),
            Bottle::new(),
        ];
        let state = GameState::new(bottles, 2);
        let solution = solve(&state);
        assert!(solution.is_some());
        let moves = solution.unwrap();
        assert!(!moves.is_empty());
    }

    #[test]
    fn test_already_solved() {
        let bottles = vec![
            Bottle::from_units(vec![1, 1, 1, 1]),
            Bottle::new(),
        ];
        let state = GameState::new(bottles, 1);
        let solution = solve(&state);
        assert!(solution.is_some());
        assert!(solution.unwrap().is_empty());
    }
}
