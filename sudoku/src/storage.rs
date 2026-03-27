use serde::{Deserialize, Serialize};
use web_sys::window;

use js_sys;

const STATS_KEY: &str = "sudoku_stats";
const HISTORY_KEY: &str = "sudoku_history";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DifficultyStats {
    pub played: u32,
    pub won: u32,
    pub best_time: Option<u32>,
    pub total_time: u32,
}

impl Default for DifficultyStats {
    fn default() -> Self {
        Self {
            played: 0,
            won: 0,
            best_time: None,
            total_time: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub easy: DifficultyStats,
    pub medium: DifficultyStats,
    pub hard: DifficultyStats,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            easy: DifficultyStats::default(),
            medium: DifficultyStats::default(),
            hard: DifficultyStats::default(),
        }
    }
}

impl Stats {
    pub fn get_mut(&mut self, difficulty: &str) -> &mut DifficultyStats {
        match difficulty {
            "easy" => &mut self.easy,
            "hard" => &mut self.hard,
            _ => &mut self.medium,
        }
    }

    pub fn get(&self, difficulty: &str) -> &DifficultyStats {
        match difficulty {
            "easy" => &self.easy,
            "hard" => &self.hard,
            _ => &self.medium,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub date: String,
    pub difficulty: String,
    pub result: String,
    pub time_secs: u32,
}

fn get_storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

pub fn load_stats() -> Stats {
    if let Some(storage) = get_storage() {
        if let Ok(Some(raw)) = storage.get_item(STATS_KEY) {
            if let Ok(stats) = serde_json::from_str(&raw) {
                return stats;
            }
        }
    }
    Stats::default()
}

pub fn save_stats(stats: &Stats) {
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
        // Keep last 100 entries
        let trimmed: &[HistoryEntry] = if history.len() > 100 {
            &history[history.len() - 100..]
        } else {
            history
        };
        if let Ok(json) = serde_json::to_string(trimmed) {
            let _ = storage.set_item(HISTORY_KEY, &json);
        }
    }
}

pub fn record_win(difficulty: &str, time_secs: u32) {
    let mut stats = load_stats();
    let s = stats.get_mut(difficulty);
    s.played += 1;
    s.won += 1;
    s.total_time += time_secs;
    if s.best_time.is_none() || time_secs < s.best_time.unwrap() {
        s.best_time = Some(time_secs);
    }
    save_stats(&stats);

    let mut history = load_history();
    history.push(HistoryEntry {
        date: now_string(),
        difficulty: difficulty.to_string(),
        result: "Won".to_string(),
        time_secs,
    });
    save_history(&history);
}

pub fn record_abandon(difficulty: &str, time_secs: u32) {
    let mut stats = load_stats();
    let s = stats.get_mut(difficulty);
    s.played += 1;
    s.total_time += time_secs;
    save_stats(&stats);

    let mut history = load_history();
    history.push(HistoryEntry {
        date: now_string(),
        difficulty: difficulty.to_string(),
        result: "Abandoned".to_string(),
        time_secs,
    });
    save_history(&history);
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
