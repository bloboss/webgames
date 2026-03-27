use yew::prelude::*;
use gloo_timers::callback::Interval;

use crate::grid::{Grid, Rng, Cell, COLS, MAX_ROWS, SCORE_MATCH, SCORE_CLEAR_BONUS, SCORE_ADD_ROW_PENALTY};
use crate::storage::{self, format_time};

pub enum Msg {
    NewGame,
    Undo,
    Hint,
    AddRow,
    ClickCell(usize),
    Tick,
    KeyDown(KeyboardEvent),
    ToggleStats,
    ToggleHistory,
}

pub struct App {
    grid: Grid,
    rng: Rng,
    score: i32,
    matches_made: u32,
    rows_added: u32,
    history: Vec<(Grid, i32, u32, u32)>,
    game_over: bool,
    game_won: bool,
    selected: Option<usize>,
    hint_a: Option<usize>,
    hint_b: Option<usize>,
    elapsed: u32,
    active: bool,
    message: String,
    message_is_win: bool,
    show_stats: bool,
    show_history: bool,
    _timer: Option<Interval>,
}

impl App {
    fn start_game(&mut self, ctx: &Context<Self>) {
        self.rng = Rng::from_entropy();
        self.grid = Grid::generate(3, &mut self.rng);
        self.score = 0;
        self.matches_made = 0;
        self.rows_added = 0;
        self.history.clear();
        self.game_over = false;
        self.game_won = false;
        self.selected = None;
        self.hint_a = None;
        self.hint_b = None;
        self.elapsed = 0;
        self.active = true;
        self.message.clear();
        self.message_is_win = false;

        let link = ctx.link().clone();
        self._timer = Some(Interval::new(1000, move || {
            link.send_message(Msg::Tick);
        }));
    }

    fn stop_timer(&mut self) {
        self._timer = None;
        self.active = false;
    }

    fn check_win(&mut self) {
        if self.grid.is_cleared() {
            self.score += SCORE_CLEAR_BONUS;
            self.game_won = true;
            self.game_over = true;
            self.stop_timer();
            storage::record_win(self.score, self.matches_made, self.rows_added, self.elapsed);
            self.message = format!(
                "Board cleared! Score: {} | Time: {}",
                self.score,
                format_time(self.elapsed)
            );
            self.message_is_win = true;
        }
    }

    fn check_no_moves(&mut self) {
        if !self.game_over && !self.grid.has_valid_match() {
            self.message = "No valid matches. Add a row or start a new game.".to_string();
            self.message_is_win = false;
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut app = App {
            grid: Grid { cells: vec![], rows: 0 },
            rng: Rng::new(1),
            score: 0,
            matches_made: 0,
            rows_added: 0,
            history: Vec::new(),
            game_over: false,
            game_won: false,
            selected: None,
            hint_a: None,
            hint_b: None,
            elapsed: 0,
            active: false,
            message: String::new(),
            message_is_win: false,
            show_stats: false,
            show_history: false,
            _timer: None,
        };
        app.start_game(ctx);
        app
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NewGame => {
                if self.active && !self.game_over {
                    storage::record_abandon(
                        self.score, self.matches_made, self.rows_added, self.elapsed,
                    );
                }
                self.start_game(ctx);
                true
            }
            Msg::Undo => {
                if let Some((grid, score, matches, rows)) = self.history.pop() {
                    self.grid = grid;
                    self.score = score;
                    self.matches_made = matches;
                    self.rows_added = rows;
                    self.game_over = false;
                    self.game_won = false;
                    self.selected = None;
                    self.hint_a = None;
                    self.hint_b = None;
                    self.message.clear();
                    self.message_is_win = false;
                    if !self.active {
                        self.active = true;
                        let link = ctx.link().clone();
                        self._timer = Some(Interval::new(1000, move || {
                            link.send_message(Msg::Tick);
                        }));
                    }
                    true
                } else {
                    self.message = "Nothing to undo.".to_string();
                    true
                }
            }
            Msg::Hint => {
                if self.game_over {
                    return false;
                }
                match self.grid.find_hint() {
                    Some((a, b)) => {
                        self.hint_a = Some(a);
                        self.hint_b = Some(b);
                        self.selected = None;
                        let (ra, ca) = self.grid.to_rc(a);
                        let (rb, cb) = self.grid.to_rc(b);
                        self.message = format!(
                            "Hint: row {} col {} & row {} col {}",
                            ra + 1, ca + 1, rb + 1, cb + 1,
                        );
                        self.message_is_win = false;
                        true
                    }
                    None => {
                        self.message = "No valid matches available.".to_string();
                        true
                    }
                }
            }
            Msg::AddRow => {
                if self.game_over {
                    return false;
                }
                if self.grid.rows >= MAX_ROWS {
                    self.message = "Maximum rows reached.".to_string();
                    self.game_over = true;
                    self.stop_timer();
                    storage::record_abandon(
                        self.score, self.matches_made, self.rows_added, self.elapsed,
                    );
                    return true;
                }

                self.history.push((
                    self.grid.clone(),
                    self.score,
                    self.matches_made,
                    self.rows_added,
                ));

                self.grid.add_row(&mut self.rng);
                self.score += SCORE_ADD_ROW_PENALTY;
                self.rows_added += 1;
                self.selected = None;
                self.hint_a = None;
                self.hint_b = None;
                self.message = format!("Row added. ({} point penalty)", -SCORE_ADD_ROW_PENALTY);
                self.message_is_win = false;
                true
            }
            Msg::ClickCell(idx) => {
                if self.game_over {
                    return false;
                }
                self.hint_a = None;
                self.hint_b = None;

                // Ignore clicks on empty cells
                if self.grid.get(idx).is_empty() {
                    self.selected = None;
                    return true;
                }

                if let Some(first) = self.selected {
                    if first == idx {
                        self.selected = None;
                        return true;
                    }
                    // Try to match
                    if self.grid.is_valid_match(first, idx) {
                        self.history.push((
                            self.grid.clone(),
                            self.score,
                            self.matches_made,
                            self.rows_added,
                        ));

                        self.grid.set(first, Cell::Empty);
                        self.grid.set(idx, Cell::Empty);
                        self.score += SCORE_MATCH;
                        self.matches_made += 1;
                        self.grid.trim_trailing_empty_rows();
                        self.selected = None;
                        self.message.clear();
                        self.message_is_win = false;
                        self.check_win();
                        if !self.game_over {
                            self.check_no_moves();
                        }
                    } else {
                        self.message = "Invalid match.".to_string();
                        self.message_is_win = false;
                        // Select the new cell instead
                        self.selected = Some(idx);
                    }
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
            Msg::KeyDown(e) => {
                let key = e.key();
                if key == "Escape" {
                    self.selected = None;
                    self.hint_a = None;
                    self.hint_b = None;
                    return true;
                }
                if key == "z" && (e.ctrl_key() || e.meta_key()) {
                    e.prevent_default();
                    ctx.link().send_message(Msg::Undo);
                }
                false
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
        let on_keydown = ctx.link().callback(Msg::KeyDown);

        html! {
            <div class="app" onkeydown={on_keydown} tabindex="0">
                <h1>{"N U M B E R   M A T C H"}</h1>

                { self.view_controls(ctx) }
                { self.view_info_bar() }
                { self.view_grid(ctx) }

                <div class={classes!("message", self.message_is_win.then_some("win"))}>
                    { &self.message }
                </div>

                { self.view_rules() }
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
                <button onclick={ctx.link().callback(|_| Msg::AddRow)}>{"Add Row (-20 pts)"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Undo)}>{"Undo"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Hint)}>{"Hint"}</button>
            </div>
        }
    }

    fn view_info_bar(&self) -> Html {
        html! {
            <div class="info-bar">
                <div class="info-item">
                    <div class="label">{"Score"}</div>
                    <div class="val">{ self.score }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Matches"}</div>
                    <div class="val">{ self.matches_made }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Remaining"}</div>
                    <div class="val">{ self.grid.remaining_count() }</div>
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
            <div class="grid" style={format!("grid-template-columns: repeat({}, 1fr)", COLS)}>
                { for (0..self.grid.total_cells()).map(|idx| {
                    self.view_cell(ctx, idx)
                })}
            </div>
        }
    }

    fn view_cell(&self, ctx: &Context<Self>, idx: usize) -> Html {
        let cell = self.grid.get(idx);
        let is_selected = self.selected == Some(idx);
        let is_hint = self.hint_a == Some(idx) || self.hint_b == Some(idx);

        match cell {
            Cell::Empty => {
                html! { <div class="cell empty"></div> }
            }
            Cell::Digit(v) => {
                let mut cls = vec!["cell"];
                cls.push(digit_class(v));
                if is_selected {
                    cls.push("selected");
                }
                if is_hint {
                    cls.push("hint");
                }
                let onclick = ctx.link().callback(move |_| Msg::ClickCell(idx));
                html! {
                    <div class={cls.join(" ")} onclick={onclick}>
                        { v }
                    </div>
                }
            }
        }
    }

    fn view_rules(&self) -> Html {
        html! {
            <div class="rules">
                <p>{"Match two numbers if they are equal or sum to 10."}</p>
                <p>{"All cells between them (in row, column, or linear order) must be empty."}</p>
            </div>
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
        let avg_time = if stats.played > 0 {
            format_time(stats.total_time / stats.played)
        } else {
            "--:--".to_string()
        };
        let best_score = stats.best_score
            .map(|s| s.to_string())
            .unwrap_or_else(|| "--".to_string());
        let best_time = stats.best_time
            .map(format_time)
            .unwrap_or_else(|| "--:--".to_string());

        html! {
            <table class="stats-table">
                <tbody>
                    <tr><td>{"Games Played"}</td><td>{ stats.played }</td></tr>
                    <tr><td>{"Games Won"}</td><td>{ stats.won }</td></tr>
                    <tr><td>{"Best Score"}</td><td>{ best_score }</td></tr>
                    <tr><td>{"Best Time"}</td><td>{ best_time }</td></tr>
                    <tr><td>{"Avg Time"}</td><td>{ avg_time }</td></tr>
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
                        <th>{"Result"}</th>
                        <th>{"Score"}</th>
                        <th>{"Matches"}</th>
                        <th>{"Rows+"}</th>
                        <th>{"Time"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for recent.iter().map(|h| {
                        let cls = if h.result == "Won" { "result-won" } else { "result-abandoned" };
                        html! {
                            <tr>
                                <td>{ &h.date }</td>
                                <td class={cls}>{ &h.result }</td>
                                <td>{ h.score }</td>
                                <td>{ h.matches }</td>
                                <td>{ h.rows_added }</td>
                                <td>{ format_time(h.time_secs) }</td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
        }
    }
}

fn digit_class(v: u8) -> &'static str {
    match v {
        1 => "d1",
        2 => "d2",
        3 => "d3",
        4 => "d4",
        5 => "d5",
        6 => "d6",
        7 => "d7",
        8 => "d8",
        9 => "d9",
        _ => "d0",
    }
}
