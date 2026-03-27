use serde::{Deserialize, Serialize};
use web_sys::window;
use js_sys;

const STATS_KEY: &str = "numbermatch_stats";
const HISTORY_KEY: &str = "numbermatch_history";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStats {
    pub played: u32,
    pub won: u32,
    pub best_score: Option<i32>,
    pub best_time: Option<u32>,
    pub total_time: u32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            played: 0,
            won: 0,
            best_score: None,
            best_time: None,
            total_time: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub date: String,
    pub result: String,
    pub score: i32,
    pub matches: u32,
    pub rows_added: u32,
    pub time_secs: u32,
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

pub fn record_win(score: i32, matches: u32, rows_added: u32, time_secs: u32) {
    let mut stats = load_stats();
    stats.played += 1;
    stats.won += 1;
    stats.total_time += time_secs;
    if stats.best_score.is_none() || score > stats.best_score.unwrap() {
        stats.best_score = Some(score);
    }
    if stats.best_time.is_none() || time_secs < stats.best_time.unwrap() {
        stats.best_time = Some(time_secs);
    }
    save_stats(&stats);

    let mut history = load_history();
    history.push(HistoryEntry {
        date: now_string(),
        result: "Won".to_string(),
        score,
        matches,
        rows_added,
        time_secs,
    });
    save_history(&history);
}

pub fn record_abandon(score: i32, matches: u32, rows_added: u32, time_secs: u32) {
    let mut stats = load_stats();
    stats.played += 1;
    stats.total_time += time_secs;
    save_stats(&stats);

    let mut history = load_history();
    history.push(HistoryEntry {
        date: now_string(),
        result: "Abandoned".to_string(),
        score,
        matches,
        rows_added,
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
