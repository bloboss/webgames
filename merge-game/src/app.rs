use yew::prelude::*;
use gloo_timers::callback::Interval;

use crate::game::{Cell, GameState, Rng, SPAWN_INTERVAL_MS, TOTAL_CELLS};
use crate::storage::{self, format_time};

pub enum Msg {
    NewGame,
    CellClick(usize),
    SpawnTick,
    TimerTick,
    ToggleStats,
    ToggleHistory,
}

pub struct App {
    game: GameState,
    rng: Rng,
    selected: Option<usize>,
    elapsed: u32,
    message: String,
    message_is_good: bool,
    show_stats: bool,
    show_history: bool,
    _spawn_timer: Option<Interval>,
    _clock_timer: Option<Interval>,
}

impl App {
    fn start_game(&mut self, ctx: &Context<Self>) {
        self.game = GameState::new();
        self.rng = Rng::from_entropy();
        self.selected = None;
        self.elapsed = 0;
        self.message.clear();
        self.message_is_good = false;

        // Spawn a few initial tiles
        for _ in 0..5 {
            self.game.spawn_tile(&mut self.rng);
        }

        self.start_timers(ctx);
        storage::save_game(&self.game, self.elapsed);
    }

    fn start_timers(&mut self, ctx: &Context<Self>) {
        let link1 = ctx.link().clone();
        self._spawn_timer = Some(Interval::new(SPAWN_INTERVAL_MS, move || {
            link1.send_message(Msg::SpawnTick);
        }));

        let link2 = ctx.link().clone();
        self._clock_timer = Some(Interval::new(1000, move || {
            link2.send_message(Msg::TimerTick);
        }));
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut app = App {
            game: GameState::new(),
            rng: Rng::from_entropy(),
            selected: None,
            elapsed: 0,
            message: String::new(),
            message_is_good: false,
            show_stats: false,
            show_history: false,
            _spawn_timer: None,
            _clock_timer: None,
        };

        if let Some(save) = storage::load_game() {
            app.game = save.game;
            app.elapsed = save.elapsed;
            app.start_timers(ctx);
        } else {
            app.start_game(ctx);
        }

        app
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NewGame => {
                storage::record_game(
                    self.game.score,
                    self.game.highest,
                    self.game.merges,
                    self.elapsed,
                );
                self.start_game(ctx);
                true
            }
            Msg::CellClick(idx) => {
                match self.game.cells[idx] {
                    Cell::Empty => {
                        self.selected = None;
                        self.message.clear();
                        true
                    }
                    Cell::Value(_) => {
                        if let Some(prev) = self.selected {
                            if prev == idx {
                                // Deselect
                                self.selected = None;
                                self.message.clear();
                            } else if self.game.can_merge(prev, idx) {
                                if let Some(new_val) = self.game.merge(prev, idx) {
                                    self.selected = None;
                                    self.message = format!("Merged into {}!", new_val);
                                    self.message_is_good = true;
                                    storage::save_game(&self.game, self.elapsed);
                                }
                            } else {
                                // Different values - move selection to newly clicked tile
                                self.selected = Some(idx);
                                self.message = "Values don't match!".to_string();
                                self.message_is_good = false;
                            }
                        } else {
                            self.selected = Some(idx);
                            self.message.clear();
                        }
                        true
                    }
                }
            }
            Msg::SpawnTick => {
                if self.game.tile_count() < crate::game::MAX_TILES {
                    self.game.spawn_tile(&mut self.rng);
                    storage::save_game(&self.game, self.elapsed);
                    true
                } else {
                    false
                }
            }
            Msg::TimerTick => {
                self.elapsed += 1;
                true
            }
            Msg::ToggleStats => {
                self.show_stats = !self.show_stats;
                true
            }
            Msg::ToggleHistory => {
                self.show_history = !self.show_history;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="app">
                <h1>{"M E R G E"}</h1>
                { self.view_controls(ctx) }
                { self.view_info_bar() }
                { self.view_grid(ctx) }

                <div class={classes!("message", self.message_is_good.then_some("good"))}>
                    { &self.message }
                </div>

                <div class="rules">
                    <p>{"Click a tile to select it, then click another tile with the same value to merge them."}</p>
                    <p>{"New tiles spawn automatically. Keep merging to reach higher values!"}</p>
                </div>

                { self.view_stats_section(ctx) }
            </div>
        }
    }
}

impl App {
    fn view_controls(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="controls">
                <button onclick={ctx.link().callback(|_| Msg::NewGame)}>{"New Game"}</button>
            </div>
        }
    }

    fn view_info_bar(&self) -> Html {
        html! {
            <div class="info-bar">
                <div class="info-item">
                    <div class="label">{"Score"}</div>
                    <div class="val">{ self.game.score }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Highest"}</div>
                    <div class="val">{ if self.game.highest > 0 { self.game.highest.to_string() } else { "--".to_string() } }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Merges"}</div>
                    <div class="val">{ self.game.merges }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Tiles"}</div>
                    <div class="val">{ self.game.tile_count() }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Time"}</div>
                    <div class="val">{ format_time(self.elapsed) }</div>
                </div>
            </div>
        }
    }

    fn view_grid(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="grid">
                { for (0..TOTAL_CELLS).map(|idx| self.view_cell(ctx, idx)) }
            </div>
        }
    }

    fn view_cell(&self, ctx: &Context<Self>, idx: usize) -> Html {
        let cell = self.game.cells[idx];
        let is_selected = self.selected == Some(idx);
        let onclick = ctx.link().callback(move |_| Msg::CellClick(idx));

        // Determine if this cell is a valid merge target for the current selection
        let is_merge_target = if let Some(sel) = self.selected {
            sel != idx && self.game.can_merge(sel, idx)
        } else {
            false
        };

        match cell {
            Cell::Empty => {
                html! {
                    <div class="cell empty" onclick={onclick}></div>
                }
            }
            Cell::Value(v) => {
                let tier = tile_tier(v);
                let mut cls = format!("cell tile t{}", tier);
                if is_selected {
                    cls.push_str(" selected");
                }
                if is_merge_target {
                    cls.push_str(" merge-target");
                }

                html! {
                    <div class={cls} onclick={onclick}>{ v }</div>
                }
            }
        }
    }

    fn view_stats_section(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="stats-section">
                <button onclick={ctx.link().callback(|_| Msg::ToggleStats)}>
                    { if self.show_stats { "Hide Stats" } else { "Show Stats" } }
                </button>
                <button onclick={ctx.link().callback(|_| Msg::ToggleHistory)}>
                    { if self.show_history { "Hide History" } else { "Show History" } }
                </button>

                { if self.show_stats { self.view_stats() } else { html! {} } }
                { if self.show_history { self.view_history() } else { html! {} } }
            </div>
        }
    }

    fn view_stats(&self) -> Html {
        let stats = storage::load_stats();

        html! {
            <table class="stats-table">
                <tbody>
                    <tr><td>{"Games Played"}</td><td>{ stats.games_played }</td></tr>
                    <tr><td>{"Best Score"}</td><td>{ stats.best_score }</td></tr>
                    <tr><td>{"Highest Tile"}</td><td>{ if stats.highest_tile > 0 { stats.highest_tile.to_string() } else { "--".to_string() } }</td></tr>
                    <tr><td>{"Total Merges"}</td><td>{ stats.total_merges }</td></tr>
                    <tr><td>{"Total Time"}</td><td>{ format_time(stats.total_time) }</td></tr>
                </tbody>
            </table>
        }
    }

    fn view_history(&self) -> Html {
        let history = storage::load_history();
        if history.is_empty() {
            return html! { <p>{"No games played yet."}</p> };
        }
        let recent: Vec<_> = history.iter().rev().take(20).collect();

        html! {
            <table class="history-table">
                <thead>
                    <tr>
                        <th>{"Date"}</th>
                        <th>{"Score"}</th>
                        <th>{"Highest"}</th>
                        <th>{"Merges"}</th>
                        <th>{"Time"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for recent.iter().map(|h| {
                        html! {
                            <tr>
                                <td>{ &h.date }</td>
                                <td>{ h.score }</td>
                                <td>{ h.highest_tile }</td>
                                <td>{ h.merges }</td>
                                <td>{ format_time(h.time_secs) }</td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
        }
    }
}

fn tile_tier(v: u32) -> u32 {
    match v {
        1 => 1,
        2 => 2,
        4 => 3,
        8 => 4,
        16 => 5,
        32 => 6,
        64 => 7,
        128 => 8,
        256 => 9,
        512 => 10,
        1024 => 11,
        _ => 12,
    }
}
