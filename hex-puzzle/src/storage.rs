use serde::{Deserialize, Serialize};
use web_sys::window;
use js_sys;

const STATS_KEY: &str = "hex_puzzle_stats";
const HISTORY_KEY: &str = "hex_puzzle_history";
const SAVE_KEY: &str = "hex_puzzle_save";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GameStats {
    pub puzzles_solved: u32,
    pub puzzles_attempted: u32,
    pub total_moves: u32,
    pub total_time: u32,
    pub best_time: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub date: String,
    pub difficulty: String,
    pub moves: u32,
    pub time_secs: u32,
    pub solved: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedGame {
    pub game: crate::game::GameState,
    pub elapsed: u32,
    pub difficulty: String,
}

fn get_storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

pub fn load_stats() -> GameStats {
    if let Some(storage) = get_storage() {
        if let Ok(Some(raw)) = storage.get_item(STATS_KEY) {
            if let Ok(stats) = serde_json::from_str(&raw) {
                return stats;
            }
        }
    }
    GameStats::default()
}

pub fn save_stats(stats: &GameStats) {
    if let Some(storage) = get_storage() {
        if let Ok(json) = serde_json::to_string(stats) {
            let _ = storage.set_item(STATS_KEY, &json);
        }
    }
}

pub fn load_history() -> Vec<HistoryEntry> {
    if let Some(storage) = get_storage() {
        if let Ok(Some(raw)) = storage.get_item(HISTORY_KEY) {
            if let Ok(history) = serde_json::from_str(&raw) {
                return history;
            }
        }
    }
    vec![]
}

fn save_history(history: &[HistoryEntry]) {
    if let Some(storage) = get_storage() {
        let trimmed = if history.len() > 100 {
            &history[history.len() - 100..]
        } else {
            history
        };
        if let Ok(json) = serde_json::to_string(trimmed) {
            let _ = storage.set_item(HISTORY_KEY, &json);
        }
    }
}

pub fn record_game(difficulty: &str, moves: u32, time_secs: u32, solved: bool) {
    let mut stats = load_stats();
    stats.puzzles_attempted += 1;
    stats.total_moves += moves;
    stats.total_time += time_secs;
    if solved {
        stats.puzzles_solved += 1;
        stats.best_time = Some(match stats.best_time {
            Some(b) => b.min(time_secs),
            None => time_secs,
        });
    }
    save_stats(&stats);

    let mut history = load_history();
    history.push(HistoryEntry {
        date: now_string(),
        difficulty: difficulty.to_string(),
        moves,
        time_secs,
        solved,
    });
    save_history(&history);
}

pub fn save_game(game: &crate::game::GameState, elapsed: u32, difficulty: &str) {
    if let Some(storage) = get_storage() {
        let save = SavedGame {
            game: game.clone(),
            elapsed,
            difficulty: difficulty.to_string(),
        };
        if let Ok(json) = serde_json::to_string(&save) {
            let _ = storage.set_item(SAVE_KEY, &json);
        }
    }
}

pub fn load_game() -> Option<SavedGame> {
    let storage = get_storage()?;
    let raw = storage.get_item(SAVE_KEY).ok()??;
    serde_json::from_str(&raw).ok()
}

pub fn clear_save() {
    if let Some(storage) = get_storage() {
        let _ = storage.remove_item(SAVE_KEY);
    }
}

fn now_string() -> String {
    let date = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}

pub fn format_time(secs: u32) -> String {
    let m = secs / 60;
    let s = secs % 60;
    format!("{:02}:{:02}", m, s)
}
