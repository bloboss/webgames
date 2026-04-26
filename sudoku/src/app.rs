use yew::prelude::*;
use gloo_timers::callback::Interval;

use crate::board::Board;
use crate::generator::{self, Difficulty};
use crate::solver;
use crate::storage::{self, format_time};

pub enum Msg {
    NewGame,
    Reset,
    Hint,
    Check,
    Solve,
    SelectCell(usize, usize),
    PlaceNumber(u8),
    SetDifficulty(&'static str),
    Tick,
    KeyDown(KeyboardEvent),
    ToggleStats,
    ToggleHistory,
    ToggleHintMode,
}

pub struct App {
    puzzle: Board,
    solution: Board,
    current: Board,
    difficulty: &'static str,
    selected: Option<(usize, usize)>,
    elapsed: u32,
    active: bool,
    message: String,
    message_is_win: bool,
    show_stats: bool,
    show_history: bool,
    _timer: Option<Interval>,
    hint_mode: bool,
    hints: [[u16; 9]; 9],
}

impl App {
    fn start_game(&mut self, ctx: &Context<Self>) {
        let diff = match self.difficulty {
            "easy" => Difficulty::Easy,
            "hard" => Difficulty::Hard,
            _ => Difficulty::Medium,
        };
        let (puzzle, solution) = generator::generate(diff);
        self.current = puzzle.clone();
        self.puzzle = puzzle;
        self.solution = solution;
        self.selected = None;
        self.elapsed = 0;
        self.active = true;
        self.message.clear();
        self.message_is_win = false;
        self.hints = [[0u16; 9]; 9];
        self.hint_mode = false;

        let link = ctx.link().clone();
        self._timer = Some(Interval::new(1000, move || {
            link.send_message(Msg::Tick);
        }));
    }

    fn stop_timer(&mut self) {
        self._timer = None;
        self.active = false;
    }

    fn error_count(&self) -> usize {
        let mut count = 0;
        for r in 0..9 {
            for c in 0..9 {
                let v = self.current.get(r, c);
                if v != 0 && v != self.solution.get(r, c) {
                    count += 1;
                }
            }
        }
        count
    }

    fn empty_count(&self) -> usize {
        let mut count = 0;
        for r in 0..9 {
            for c in 0..9 {
                if self.current.is_empty(r, c) {
                    count += 1;
                }
            }
        }
        count
    }

    fn check_win(&mut self) {
        if solver::verify(&self.current) {
            self.stop_timer();
            storage::record_win(self.difficulty, self.elapsed);
            self.message = format!(
                "Puzzle Complete! Time: {}",
                format_time(self.elapsed)
            );
            self.message_is_win = true;
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut app = App {
            puzzle: Board::new(),
            solution: Board::new(),
            current: Board::new(),
            difficulty: "easy",
            selected: None,
            elapsed: 0,
            active: false,
            message: String::new(),
            message_is_win: false,
            show_stats: false,
            show_history: false,
            _timer: None,
            hint_mode: false,
            hints: [[0u16; 9]; 9],
        };
        app.start_game(ctx);
        app
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NewGame => {
                if self.active {
                    storage::record_abandon(self.difficulty, self.elapsed);
                }
                self.start_game(ctx);
                true
            }
            Msg::Reset => {
                self.current = self.puzzle.clone();
                self.selected = None;
                self.message = "Board reset.".to_string();
                self.message_is_win = false;
                self.hints = [[0u16; 9]; 9];
                // Restart timer
                self.elapsed = 0;
                self.active = true;
                let link = ctx.link().clone();
                self._timer = Some(Interval::new(1000, move || {
                    link.send_message(Msg::Tick);
                }));
                true
            }
            Msg::Hint => {
                if !self.active {
                    return false;
                }
                for r in 0..9 {
                    for c in 0..9 {
                        if self.current.is_empty(r, c) {
                            let val = self.solution.get(r, c);
                            self.current.set(r, c, val);
                            self.hints[r][c] = 0;
                            self.selected = Some((r, c));
                            self.message = format!(
                                "Hint: {} at row {}, col {}",
                                val,
                                r + 1,
                                c + 1
                            );
                            self.message_is_win = false;
                            self.check_win();
                            return true;
                        }
                    }
                }
                self.message = "No empty cells!".to_string();
                true
            }
            Msg::Check => {
                let errors = self.error_count();
                if errors == 0 {
                    self.message = "Looking good! No errors found.".to_string();
                } else {
                    self.message = format!("Found {} error(s).", errors);
                }
                self.message_is_win = false;
                true
            }
            Msg::Solve => {
                self.stop_timer();
                for r in 0..9 {
                    for c in 0..9 {
                        if self.puzzle.is_empty(r, c) {
                            self.current.set(r, c, self.solution.get(r, c));
                            self.hints[r][c] = 0;
                        }
                    }
                }
                self.message = "Solved. Start a new game to play again.".to_string();
                self.message_is_win = false;
                true
            }
            Msg::SelectCell(r, c) => {
                if self.active {
                    self.selected = Some((r, c));
                    true
                } else {
                    false
                }
            }
            Msg::PlaceNumber(num) => {
                if !self.active {
                    return false;
                }
                if let Some((r, c)) = self.selected {
                    if self.puzzle.is_empty(r, c) {
                        if self.hint_mode {
                            if self.current.is_empty(r, c) {
                                if num == 0 {
                                    self.hints[r][c] = 0;
                                } else {
                                    self.hints[r][c] ^= 1 << num;
                                }
                                return true;
                            }
                            return false;
                        } else {
                            self.current.set(r, c, num);
                            self.hints[r][c] = 0;
                            self.message.clear();
                            self.message_is_win = false;
                            if num != 0 {
                                self.check_win();
                            }
                            return true;
                        }
                    }
                }
                false
            }
            Msg::SetDifficulty(d) => {
                self.difficulty = d;
                true
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
                if !self.active {
                    return false;
                }
                let key = e.key();
                match key.as_str() {
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                        let num: u8 = key.parse().unwrap();
                        ctx.link().send_message(Msg::PlaceNumber(num));
                        false
                    }
                    "Backspace" | "Delete" | "0" => {
                        ctx.link().send_message(Msg::PlaceNumber(0));
                        false
                    }
                    "ArrowUp" => {
                        if let Some((r, c)) = self.selected {
                            if r > 0 {
                                self.selected = Some((r - 1, c));
                                return true;
                            }
                        }
                        false
                    }
                    "ArrowDown" => {
                        if let Some((r, c)) = self.selected {
                            if r < 8 {
                                self.selected = Some((r + 1, c));
                                return true;
                            }
                        }
                        false
                    }
                    "ArrowLeft" => {
                        if let Some((r, c)) = self.selected {
                            if c > 0 {
                                self.selected = Some((r, c - 1));
                                return true;
                            }
                        }
                        false
                    }
                    "ArrowRight" => {
                        if let Some((r, c)) = self.selected {
                            if c < 8 {
                                self.selected = Some((r, c + 1));
                                return true;
                            }
                        }
                        false
                    }
                    _ => false,
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
            Msg::ToggleHintMode => {
                self.hint_mode = !self.hint_mode;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_keydown = ctx.link().callback(Msg::KeyDown);

        html! {
            <div class="app" onkeydown={on_keydown} tabindex="0">
                <h1>{"S U D O K U"}</h1>

                { self.view_controls(ctx) }

                <div class="game-area">
                    { self.view_board(ctx) }
                    { self.view_sidebar(ctx) }
                </div>

                <div class={classes!(
                    "message",
                    self.message_is_win.then_some("win")
                )}>
                    { &self.message }
                </div>

                { self.view_stats_section(ctx) }
            </div>
        }
    }
}

impl App {
    fn view_controls(&self, ctx: &Context<Self>) -> Html {
        let difficulties: Vec<(&str, &str)> =
            vec![("easy", "Easy"), ("medium", "Medium"), ("hard", "Hard")];

        html! {
            <div class="controls">
                { for difficulties.iter().map(|(val, label)| {
                    let v = *val;
                    let active = self.difficulty == v;
                    let onclick = ctx.link().callback(move |_| Msg::SetDifficulty(
                        match v { "easy" => "easy", "hard" => "hard", _ => "medium" }
                    ));
                    html! {
                        <button class={classes!(active.then_some("active"))}
                                onclick={onclick}>
                            { label }
                        </button>
                    }
                })}
                <button onclick={ctx.link().callback(|_| Msg::NewGame)}>{"New Game"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Reset)}>{"Reset"}</button>
                <button class={classes!(self.hint_mode.then_some("active"))}
                        onclick={ctx.link().callback(|_| Msg::ToggleHintMode)}>
                    { if self.hint_mode { "Notes ON" } else { "Notes" } }
                </button>
                <button onclick={ctx.link().callback(|_| Msg::Hint)}>{"Hint"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Check)}>{"Check"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Solve)}>{"Solve"}</button>
            </div>
        }
    }

    fn view_board(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="board">
                { for (0..9).map(|r| {
                    html! { <>
                        { for (0..9).map(|c| {
                            self.view_cell(ctx, r, c)
                        })}
                    </> }
                })}
            </div>
        }
    }

    fn view_cell(&self, ctx: &Context<Self>, r: usize, c: usize) -> Html {
        let pv = self.puzzle.get(r, c);
        let cv = self.current.get(r, c);
        let sv = self.solution.get(r, c);

        let is_selected = self.selected == Some((r, c));
        let is_highlighted = if let Some((sr, sc)) = self.selected {
            r == sr || c == sc || (r / 3 == sr / 3 && c / 3 == sc / 3)
        } else {
            false
        };

        let mut classes = vec!["cell"];

        if is_selected {
            classes.push("selected");
        } else if is_highlighted {
            classes.push("highlighted");
        }

        // Thicker borders for 3x3 boxes
        if c % 3 == 2 && c != 8 {
            classes.push("border-right");
        }
        if r % 3 == 2 && r != 8 {
            classes.push("border-bottom");
        }

        let onclick = ctx.link().callback(move |_| Msg::SelectCell(r, c));

        if pv != 0 {
            classes.push("given");
            html! {
                <div class={classes.join(" ")} onclick={onclick}>
                    { pv.to_string() }
                </div>
            }
        } else if cv != 0 {
            if cv != sv {
                classes.push("error");
            } else {
                classes.push("user");
            }
            html! {
                <div class={classes.join(" ")} onclick={onclick}>
                    { cv.to_string() }
                </div>
            }
        } else {
            let mask = self.hints[r][c];
            if mask != 0 {
                classes.push("hints");
                html! {
                    <div class={classes.join(" ")} onclick={onclick}>
                        { Self::view_hints_grid(mask) }
                    </div>
                }
            } else {
                classes.push("empty");
                html! {
                    <div class={classes.join(" ")} onclick={onclick}>
                    </div>
                }
            }
        }
    }

    fn view_hints_grid(mask: u16) -> Html {
        html! {
            <div class="hints-grid">
                { for (1u8..=9).map(|n| {
                    let visible = (mask >> n) & 1 == 1;
                    html! {
                        <span class="hint-num">
                            { if visible { n.to_string() } else { String::new() } }
                        </span>
                    }
                })}
            </div>
        }
    }

    fn view_sidebar(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="sidebar">
                <div class="info-box">
                    <div class="label">{"Time"}</div>
                    <div class="value">{ format_time(self.elapsed) }</div>
                </div>
                <div class="info-box">
                    <div class="label">{"Errors"}</div>
                    <div class="value small">{ self.error_count() }</div>
                </div>
                <div class="info-box">
                    <div class="label">{"Remaining"}</div>
                    <div class="value small">{ self.empty_count() }</div>
                </div>
                <div class="numpad">
                    { for (1..=9).map(|n| {
                        let onclick = ctx.link().callback(move |_| Msg::PlaceNumber(n));
                        html! {
                            <button onclick={onclick}>{ n }</button>
                        }
                    })}
                    <button class="clear-btn"
                            onclick={ctx.link().callback(|_| Msg::PlaceNumber(0))}>
                        {"Clear"}
                    </button>
                </div>
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
        let diffs = ["easy", "medium", "hard"];

        html! {
            <table class="stats-table">
                <thead>
                    <tr>
                        <th>{"Difficulty"}</th>
                        <th>{"Played"}</th>
                        <th>{"Won"}</th>
                        <th>{"Best Time"}</th>
                        <th>{"Avg Time"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for diffs.iter().map(|d| {
                        let s = stats.get(d);
                        let best = s.best_time.map(format_time).unwrap_or_else(|| "--:--".to_string());
                        let avg = if s.won > 0 {
                            format_time(s.total_time / s.won)
                        } else {
                            "--:--".to_string()
                        };
                        let label = match *d {
                            "easy" => "Easy",
                            "medium" => "Medium",
                            "hard" => "Hard",
                            _ => d,
                        };
                        html! {
                            <tr>
                                <td>{ label }</td>
                                <td>{ s.played }</td>
                                <td>{ s.won }</td>
                                <td>{ best }</td>
                                <td>{ avg }</td>
                            </tr>
                        }
                    })}
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
                        <th>{"Difficulty"}</th>
                        <th>{"Result"}</th>
                        <th>{"Time"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for recent.iter().map(|h| {
                        let cls = if h.result == "Won" { "result-won" } else { "result-abandoned" };
                        html! {
                            <tr>
                                <td>{ &h.date }</td>
                                <td>{ &h.difficulty }</td>
                                <td class={cls}>{ &h.result }</td>
                                <td>{ format_time(h.time_secs) }</td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
        }
    }
}
