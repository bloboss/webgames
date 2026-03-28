use yew::prelude::*;
use gloo_timers::callback::Interval;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, TouchEvent};

use crate::board::{Board, Cell, Direction, Rng, TOTAL_CELLS};
use crate::storage::{self, format_time};

pub enum Msg {
    NewGame,
    Undo,
    KeyDown(KeyboardEvent),
    TouchStart(TouchEvent),
    TouchEnd(TouchEvent),
    Tick,
    ToggleStats,
    ToggleHistory,
}

pub struct App {
    board: Board,
    prev_boards: Vec<(Board, u32)>,
    rng: Rng,
    elapsed: u32,
    active: bool,
    game_over: bool,
    won: bool,
    keep_playing: bool,
    message: String,
    message_is_good: bool,
    show_stats: bool,
    show_history: bool,
    touch_start: Option<(f64, f64)>,
    _timer: Option<Interval>,
    _key_listener: Option<Closure<dyn Fn(KeyboardEvent)>>,
}

impl App {
    fn start_game(&mut self, ctx: &Context<Self>) {
        self.board = Board::new();
        self.rng = Rng::from_entropy();
        self.prev_boards.clear();
        self.elapsed = 0;
        self.active = true;
        self.game_over = false;
        self.won = false;
        self.keep_playing = false;
        self.message.clear();
        self.message_is_good = false;
        self.touch_start = None;

        // Spawn 2 initial tiles (standard 2048)
        for _ in 0..2 {
            self.spawn_tile();
        }

        let link = ctx.link().clone();
        self._timer = Some(Interval::new(1000, move || {
            link.send_message(Msg::Tick);
        }));

        storage::save_game(&self.board, self.elapsed);
    }

    fn spawn_tile(&mut self) {
        let empty = self.board.empty_indices();
        if empty.is_empty() {
            return;
        }
        let idx = empty[self.rng.range(empty.len())];
        // 90% chance of 2, 10% chance of 4 (standard 2048 probabilities)
        let val = if self.rng.range(10) == 0 { 4 } else { 2 };
        self.board.place(idx, val);
    }

    fn stop_timer(&mut self) {
        self._timer = None;
        self.active = false;
    }

    fn check_game_over(&mut self) {
        if self.board.is_game_over() {
            self.game_over = true;
            self.stop_timer();
            storage::record_game(
                self.board.score,
                self.board.highest,
                self.board.merges,
                self.elapsed,
            );
            storage::clear_save();
            self.message = format!(
                "Game Over! Score: {} | Highest: {}",
                self.board.score, self.board.highest
            );
            self.message_is_good = false;
        }
    }

    fn check_win(&mut self) {
        if !self.won && !self.keep_playing && self.board.highest >= 2048 {
            self.won = true;
            self.message = "You reached 2048! Press any arrow key to keep playing.".to_string();
            self.message_is_good = true;
        }
    }

    fn do_move(&mut self, dir: Direction) -> bool {
        if self.game_over {
            return false;
        }

        // If won but not yet acknowledged, mark as keep_playing
        if self.won && !self.keep_playing {
            self.keep_playing = true;
            self.message.clear();
        }

        self.prev_boards.push((self.board.clone(), self.elapsed));
        let changed = self.board.slide(dir);

        if changed {
            self.spawn_tile();
            storage::save_game(&self.board, self.elapsed);
            self.check_win();
            self.check_game_over();
            true
        } else {
            // No change, remove the undo entry
            self.prev_boards.pop();
            false
        }
    }

    fn setup_key_listener(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let closure = Closure::wrap(Box::new(move |e: KeyboardEvent| {
            link.send_message(Msg::KeyDown(e));
        }) as Box<dyn Fn(KeyboardEvent)>);

        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "keydown",
                closure.as_ref().unchecked_ref(),
            );
        }
        self._key_listener = Some(closure);
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut app = App {
            board: Board::new(),
            prev_boards: Vec::new(),
            rng: Rng::from_entropy(),
            elapsed: 0,
            active: false,
            game_over: false,
            won: false,
            keep_playing: false,
            message: String::new(),
            message_is_good: false,
            show_stats: false,
            show_history: false,
            touch_start: None,
            _timer: None,
            _key_listener: None,
        };

        // Try to restore saved game
        if let Some(save) = storage::load_game() {
            app.board = save.board;
            app.elapsed = save.elapsed;
            app.active = true;
            let link = ctx.link().clone();
            app._timer = Some(Interval::new(1000, move || {
                link.send_message(Msg::Tick);
            }));

            if app.board.is_game_over() {
                app.game_over = true;
                app.active = false;
                app._timer = None;
                app.message = "Game Over! Start a new game.".to_string();
            }

            // Check if already won
            if app.board.highest >= 2048 {
                app.won = true;
                app.keep_playing = true;
            }
        } else {
            app.start_game(ctx);
        }

        app.setup_key_listener(ctx);
        app
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NewGame => {
                if self.active && !self.game_over {
                    storage::record_game(
                        self.board.score,
                        self.board.highest,
                        self.board.merges,
                        self.elapsed,
                    );
                }
                self.start_game(ctx);
                true
            }
            Msg::Undo => {
                if let Some((board, elapsed)) = self.prev_boards.pop() {
                    self.board = board;
                    self.elapsed = elapsed;
                    self.game_over = false;
                    self.message.clear();
                    self.message_is_good = false;
                    if !self.active {
                        self.active = true;
                        let link = ctx.link().clone();
                        self._timer = Some(Interval::new(1000, move || {
                            link.send_message(Msg::Tick);
                        }));
                    }
                    storage::save_game(&self.board, self.elapsed);
                    true
                } else {
                    self.message = "Nothing to undo.".to_string();
                    self.message_is_good = false;
                    true
                }
            }
            Msg::KeyDown(e) => {
                let dir = match e.key().as_str() {
                    "ArrowUp" | "w" | "W" => Some(Direction::Up),
                    "ArrowDown" | "s" | "S" => Some(Direction::Down),
                    "ArrowLeft" | "a" | "A" => Some(Direction::Left),
                    "ArrowRight" | "d" | "D" => Some(Direction::Right),
                    _ => None,
                };
                if let Some(dir) = dir {
                    e.prevent_default();
                    self.do_move(dir);
                    true
                } else {
                    false
                }
            }
            Msg::TouchStart(e) => {
                if let Some(touch) = e.changed_touches().get(0) {
                    self.touch_start = Some((touch.client_x() as f64, touch.client_y() as f64));
                }
                false
            }
            Msg::TouchEnd(e) => {
                if let Some((sx, sy)) = self.touch_start.take() {
                    if let Some(touch) = e.changed_touches().get(0) {
                        let dx = touch.client_x() as f64 - sx;
                        let dy = touch.client_y() as f64 - sy;
                        let min_swipe = 30.0;

                        if dx.abs() > dy.abs() && dx.abs() > min_swipe {
                            if dx > 0.0 {
                                self.do_move(Direction::Right);
                            } else {
                                self.do_move(Direction::Left);
                            }
                            return true;
                        } else if dy.abs() > dx.abs() && dy.abs() > min_swipe {
                            if dy > 0.0 {
                                self.do_move(Direction::Down);
                            } else {
                                self.do_move(Direction::Up);
                            }
                            return true;
                        }
                    }
                }
                false
            }
            Msg::Tick => {
                if self.active {
                    self.elapsed += 1;
                    true
                } else {
                    false
                }
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
        let ontouchstart = ctx.link().callback(Msg::TouchStart);
        let ontouchend = ctx.link().callback(Msg::TouchEnd);

        html! {
            <div class="app"
                 ontouchstart={ontouchstart}
                 ontouchend={ontouchend}>
                <h1>{"2 0 4 8"}</h1>

                { self.view_controls(ctx) }
                { self.view_info_bar() }
                { self.view_grid() }

                <div class={classes!("message", self.message_is_good.then_some("good"))}>
                    { &self.message }
                </div>

                <div class="rules">
                    <p>{"Use arrow keys or WASD to slide tiles. Swipe on mobile."}</p>
                    <p>{"Tiles with the same value merge when they collide. Reach 2048 to win!"}</p>
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
                <button onclick={ctx.link().callback(|_| Msg::Undo)}>{"Undo"}</button>
            </div>
        }
    }

    fn view_info_bar(&self) -> Html {
        html! {
            <div class="info-bar">
                <div class="info-item">
                    <div class="label">{"Score"}</div>
                    <div class="val">{ self.board.score }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Highest"}</div>
                    <div class="val">{ if self.board.highest > 0 { self.board.highest.to_string() } else { "--".to_string() } }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Merges"}</div>
                    <div class="val">{ self.board.merges }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Time"}</div>
                    <div class="val">{ format_time(self.elapsed) }</div>
                </div>
            </div>
        }
    }

    fn view_grid(&self) -> Html {
        html! {
            <div class="grid">
                { for (0..TOTAL_CELLS).map(|idx| self.view_cell(idx)) }
            </div>
        }
    }

    fn view_cell(&self, idx: usize) -> Html {
        let cell = self.board.cells[idx];

        match cell {
            Cell::Empty => {
                html! {
                    <div class="cell empty"></div>
                }
            }
            Cell::Value(v) => {
                let tier = tile_tier(v);
                let tier_class = format!("cell tile t{}", tier);

                html! {
                    <div class={tier_class}>
                        { v }
                    </div>
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

/// Map a tile value to a tier number for color styling.
fn tile_tier(v: u32) -> u32 {
    match v {
        2 => 1,
        4 => 2,
        8 => 3,
        16 => 4,
        32 => 5,
        64 => 6,
        128 => 7,
        256 => 8,
        512 => 9,
        1024 => 10,
        2048 => 11,
        _ => 12,
    }
}
