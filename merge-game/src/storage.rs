use serde::{Deserialize, Serialize};
use web_sys::window;
use js_sys;

const STATS_KEY: &str = "merge_stats";
const HISTORY_KEY: &str = "merge_history";
const SAVE_KEY: &str = "merge_save";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStats {
    pub games_played: u32,
    pub best_score: u64,
    pub highest_tile: u32,
    pub total_merges: u32,
    pub total_time: u32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            games_played: 0,
            best_score: 0,
            highest_tile: 0,
            total_merges: 0,
            total_time: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub date: String,
    pub score: u64,
    pub highest_tile: u32,
    pub merges: u32,
    pub time_secs: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedGame {
    pub board: crate::board::Board,
    pub elapsed: u32,
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

pub fn save_history(history: &[HistoryEntry]) {
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

pub fn record_game(score: u64, highest_tile: u32, merges: u32, time_secs: u32) {
    let mut stats = load_stats();
    stats.games_played += 1;
    stats.total_merges += merges;
    stats.total_time += time_secs;
    if score > stats.best_score {
        stats.best_score = score;
    }
    if highest_tile > stats.highest_tile {
        stats.highest_tile = highest_tile;
    }
    save_stats(&stats);

    let mut history = load_history();
    history.push(HistoryEntry {
        date: now_string(),
        score,
        highest_tile,
        merges,
        time_secs,
    });
    save_history(&history);
}

pub fn save_game(board: &crate::board::Board, elapsed: u32) {
    if let Some(storage) = get_storage() {
        let save = SavedGame {
            board: board.clone(),
            elapsed,
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
