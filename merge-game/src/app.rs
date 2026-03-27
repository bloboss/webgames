use yew::prelude::*;
use gloo_timers::callback::Interval;
use web_sys::DragEvent;

use crate::board::{Board, Cell, Rng, TOTAL_CELLS};
use crate::storage::{self, format_time};

pub enum Msg {
    NewGame,
    Undo,
    DragStart(usize),
    DragOver(DragEvent),
    Drop(usize),
    Tick,
    ToggleStats,
    ToggleHistory,
    /// Click-based merge: first click selects, second click on matching tile merges
    ClickCell(usize),
}

pub struct App {
    board: Board,
    prev_boards: Vec<(Board, u32)>,
    rng: Rng,
    elapsed: u32,
    active: bool,
    game_over: bool,
    dragging: Option<usize>,
    selected: Option<usize>,
    message: String,
    message_is_good: bool,
    show_stats: bool,
    show_history: bool,
    _timer: Option<Interval>,
}

impl App {
    fn start_game(&mut self, ctx: &Context<Self>) {
        self.board = Board::new();
        self.rng = Rng::from_entropy();
        self.prev_boards.clear();
        self.elapsed = 0;
        self.active = true;
        self.game_over = false;
        self.dragging = None;
        self.selected = None;
        self.message.clear();
        self.message_is_good = false;

        // Spawn 3 initial tiles
        for _ in 0..3 {
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
        // 80% chance of 2, 20% chance of 4
        let val = if self.rng.range(5) == 0 { 4 } else { 2 };
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

    fn do_merge(&mut self, src: usize, dst: usize) -> bool {
        if !self.board.can_merge(src, dst) {
            return false;
        }

        self.prev_boards.push((self.board.clone(), self.elapsed));
        let merged = self.board.merge(src, dst).unwrap();
        self.spawn_tile();
        self.message = format!("Merged into {}!", merged);
        self.message_is_good = true;

        storage::save_game(&self.board, self.elapsed);
        self.check_game_over();
        true
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
            dragging: None,
            selected: None,
            message: String::new(),
            message_is_good: false,
            show_stats: false,
            show_history: false,
            _timer: None,
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
        } else {
            app.start_game(ctx);
        }
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
                    self.selected = None;
                    self.dragging = None;
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
            Msg::DragStart(idx) => {
                if self.game_over {
                    return false;
                }
                if !self.board.cells[idx].is_empty() {
                    self.dragging = Some(idx);
                    self.selected = None;
                }
                false
            }
            Msg::DragOver(e) => {
                e.prevent_default();
                false
            }
            Msg::Drop(dst) => {
                if let Some(src) = self.dragging.take() {
                    if self.do_merge(src, dst) {
                        return true;
                    }
                    self.message = "Can only merge tiles with the same value.".to_string();
                    self.message_is_good = false;
                }
                true
            }
            Msg::ClickCell(idx) => {
                if self.game_over {
                    return false;
                }
                if self.board.cells[idx].is_empty() {
                    self.selected = None;
                    return true;
                }
                if let Some(src) = self.selected {
                    if src == idx {
                        self.selected = None;
                        return true;
                    }
                    if self.do_merge(src, idx) {
                        self.selected = None;
                        return true;
                    }
                    // Not a valid merge — select the new cell instead
                    self.selected = Some(idx);
                    self.message = "Can only merge tiles with the same value.".to_string();
                    self.message_is_good = false;
                    true
                } else {
                    self.selected = Some(idx);
                    self.message.clear();
                    true
                }
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
                    <p>{"Drag or click a tile onto another tile with the same value to merge them."}</p>
                    <p>{"Merged tiles double in value. A new tile spawns after each merge."}</p>
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

    fn view_grid(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="grid">
                { for (0..TOTAL_CELLS).map(|idx| self.view_cell(ctx, idx)) }
            </div>
        }
    }

    fn view_cell(&self, ctx: &Context<Self>, idx: usize) -> Html {
        let cell = self.board.cells[idx];
        let is_selected = self.selected == Some(idx);

        match cell {
            Cell::Empty => {
                let ondragover = ctx.link().callback(Msg::DragOver);
                let ondrop = ctx.link().callback(move |e: DragEvent| {
                    e.prevent_default();
                    Msg::Drop(idx)
                });
                html! {
                    <div class="cell empty"
                         ondragover={ondragover}
                         ondrop={ondrop}>
                    </div>
                }
            }
            Cell::Value(v) => {
                let tier = tile_tier(v);
                let mut classes = vec!["cell", "tile"];
                let tier_class = format!("t{}", tier);
                if is_selected {
                    classes.push("selected");
                }

                let ondragstart = ctx.link().callback(move |_: DragEvent| Msg::DragStart(idx));
                let ondragover = ctx.link().callback(Msg::DragOver);
                let ondrop = ctx.link().callback(move |e: DragEvent| {
                    e.prevent_default();
                    Msg::Drop(idx)
                });
                let onclick = ctx.link().callback(move |_| Msg::ClickCell(idx));

                html! {
                    <div class={format!("{} {}", classes.join(" "), tier_class)}
                         draggable="true"
                         ondragstart={ondragstart}
                         ondragover={ondragover}
                         ondrop={ondrop}
                         onclick={onclick}>
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
