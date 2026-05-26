use yew::prelude::*;
use gloo_timers::callback::Interval;

use crate::generator::{self, Difficulty};
use crate::multi_board::{MultiBoard, GRID_SIZE, BOARD_OFFSETS};
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
}

pub struct App {
    puzzle: MultiBoard,
    solution: MultiBoard,
    current: MultiBoard,
    difficulty: &'static str,
    selected: Option<(usize, usize)>,
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
        if self.current.is_solved() {
            self.stop_timer();
            storage::record_win(self.difficulty, self.elapsed);
            self.message = format!("All boards complete! Time: {}", format_time(self.elapsed));
            self.message_is_win = true;
        }
    }

    /// Find the next active cell in a given direction from (gr, gc)
    fn next_active(gr: usize, gc: usize, dr: i32, dc: i32) -> Option<(usize, usize)> {
        let mut r = gr as i32 + dr;
        let mut c = gc as i32 + dc;
        while r >= 0 && r < GRID_SIZE as i32 && c >= 0 && c < GRID_SIZE as i32 {
            if MultiBoard::is_active(r as usize, c as usize) {
                return Some((r as usize, c as usize));
            }
            r += dr;
            c += dc;
        }
        None
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut app = App {
            puzzle: MultiBoard::new(),
            solution: MultiBoard::new(),
            current: MultiBoard::new(),
            difficulty: "easy",
            selected: None,
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
                for gr in 0..GRID_SIZE {
                    for gc in 0..GRID_SIZE {
                        if MultiBoard::is_active(gr, gc) && self.current.is_empty(gr, gc) {
                            let val = self.solution.get(gr, gc);
                            self.current.set(gr, gc, val);
                            self.selected = Some((gr, gc));
                            self.message = format!("Hint: {} placed", val);
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
                let errors = self.current.error_count(&self.solution);
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
                for gr in 0..GRID_SIZE {
                    for gc in 0..GRID_SIZE {
                        if MultiBoard::is_active(gr, gc) && self.puzzle.is_empty(gr, gc) {
                            self.current.set(gr, gc, self.solution.get(gr, gc));
                        }
                    }
                }
                self.message = "Solved. Start a new game to play again.".to_string();
                self.message_is_win = false;
                true
            }
            Msg::SelectCell(gr, gc) => {
                if self.active && MultiBoard::is_active(gr, gc) {
                    self.selected = Some((gr, gc));
                    true
                } else {
                    false
                }
            }
            Msg::PlaceNumber(num) => {
                if !self.active {
                    return false;
                }
                if let Some((gr, gc)) = self.selected {
                    if MultiBoard::is_active(gr, gc) && self.puzzle.is_empty(gr, gc) {
                        self.current.set(gr, gc, num);
                        self.message.clear();
                        self.message_is_win = false;
                        if num != 0 {
                            self.check_win();
                        }
                        return true;
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
                        ctx.link()
                            .send_message(Msg::PlaceNumber(key.parse().unwrap()));
                        false
                    }
                    "Backspace" | "Delete" | "0" => {
                        ctx.link().send_message(Msg::PlaceNumber(0));
                        false
                    }
                    "ArrowUp" => {
                        if let Some((gr, gc)) = self.selected {
                            if let Some(next) = Self::next_active(gr, gc, -1, 0) {
                                self.selected = Some(next);
                                return true;
                            }
                        }
                        false
                    }
                    "ArrowDown" => {
                        if let Some((gr, gc)) = self.selected {
                            if let Some(next) = Self::next_active(gr, gc, 1, 0) {
                                self.selected = Some(next);
                                return true;
                            }
                        }
                        false
                    }
                    "ArrowLeft" => {
                        if let Some((gr, gc)) = self.selected {
                            if let Some(next) = Self::next_active(gr, gc, 0, -1) {
                                self.selected = Some(next);
                                return true;
                            }
                        }
                        false
                    }
                    "ArrowRight" => {
                        if let Some((gr, gc)) = self.selected {
                            if let Some(next) = Self::next_active(gr, gc, 0, 1) {
                                self.selected = Some(next);
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_keydown = ctx.link().callback(Msg::KeyDown);

        html! {
            <div class="app" onkeydown={on_keydown} tabindex="0">
                <h1>{"M U L T I - S U D O K U"}</h1>

                { self.view_controls(ctx) }

                <div class="game-area">
                    { self.view_grid(ctx) }
                    { self.view_sidebar(ctx) }
                </div>

                { self.view_number_bar(ctx) }

                <div class={classes!("message", self.message_is_win.then_some("win"))}>
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
                <button onclick={ctx.link().callback(|_| Msg::Hint)}>{"Hint"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Check)}>{"Check"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Solve)}>{"Solve"}</button>
            </div>
        }
    }

    fn view_grid(&self, ctx: &Context<Self>) -> Html {
        // Render 21x21 grid; inactive cells are invisible spacers.
        // Cell size and grid dimensions exposed as CSS vars so the mobile
        // stylesheet can scale cells to the viewport.
        html! {
            <div class="multi-grid" style={format!(
                "--grid-cols: {}; --grid-rows: {};", GRID_SIZE, GRID_SIZE
            )}>
                { for (0..GRID_SIZE).flat_map(|gr| {
                    (0..GRID_SIZE).map(move |gc| (gr, gc))
                }).map(|(gr, gc)| {
                    self.view_cell(ctx, gr, gc)
                })}
            </div>
        }
    }

    fn view_cell(&self, ctx: &Context<Self>, gr: usize, gc: usize) -> Html {
        if !MultiBoard::is_active(gr, gc) {
            return html! {
                <div class="cell-spacer"></div>
            };
        }

        let pv = self.puzzle.get(gr, gc);
        let cv = self.current.get(gr, gc);
        let sv = self.solution.get(gr, gc);
        let is_overlap = MultiBoard::is_overlap(gr, gc);

        let is_selected = self.selected == Some((gr, gc));

        // Highlight same row/col/box within the same board(s)
        let is_highlighted = if let Some((sr, sc)) = self.selected {
            let sel_boards = MultiBoard::owning_boards(sr, sc);
            let cell_boards = MultiBoard::owning_boards(gr, gc);
            // Check if they share a board and are in same row/col/box within it
            sel_boards.iter().any(|&sb| {
                cell_boards.iter().any(|&cb| {
                    if sb != cb {
                        return false;
                    }
                    let (slr, slc) = MultiBoard::grid_to_local(sb, sr, sc);
                    let (clr, clc) = MultiBoard::grid_to_local(cb, gr, gc);
                    slr == clr
                        || slc == clc
                        || (slr / 3 == clr / 3 && slc / 3 == clc / 3)
                })
            })
        } else {
            false
        };

        let mut classes = vec!["cell"];
        if is_selected {
            classes.push("selected");
        } else if is_highlighted {
            classes.push("highlighted");
        }
        if is_overlap {
            classes.push("overlap");
        }

        // Determine thick borders for 3x3 box boundaries
        // A cell needs a thick right border if, within any owning board,
        // its local column is at position 2 or 5 (end of a 3x3 box) and not at col 8
        let owners = MultiBoard::owning_boards(gr, gc);
        let mut border_right = false;
        let mut border_bottom = false;
        for &b in &owners {
            let (lr, lc) = MultiBoard::grid_to_local(b, gr, gc);
            // Right border: at box boundary, and the cell to the right is also in this board
            if lc % 3 == 2 && lc != 8 {
                let (or, oc) = BOARD_OFFSETS[b];
                if gc + 1 < or + 9 {
                    border_right = true;
                }
            }
            if lr % 3 == 2 && lr != 8 {
                let (or, _) = BOARD_OFFSETS[b];
                if gr + 1 < or + 9 {
                    border_bottom = true;
                }
            }
            // Board edge borders
            if lc == 8 {
                border_right = true;
            }
            if lr == 8 {
                border_bottom = true;
            }
            if lc == 0 {
                classes.push("border-left-thick");
            }
            if lr == 0 {
                classes.push("border-top-thick");
            }
        }

        if border_right {
            classes.push("border-right");
        }
        if border_bottom {
            classes.push("border-bottom");
        }

        let display = if pv != 0 {
            classes.push("given");
            pv.to_string()
        } else if cv != 0 {
            if cv != sv {
                classes.push("error");
            } else {
                classes.push("user");
            }
            cv.to_string()
        } else {
            classes.push("empty");
            String::new()
        };

        let onclick = ctx.link().callback(move |_| Msg::SelectCell(gr, gc));

        html! {
            <div class={classes.join(" ")} onclick={onclick}>
                { display }
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
                    <div class="value small">{ self.current.error_count(&self.solution) }</div>
                </div>
                <div class="info-box">
                    <div class="label">{"Remaining"}</div>
                    <div class="value small">{ self.current.empty_count() }</div>
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

    fn view_number_bar(&self, ctx: &Context<Self>) -> Html {
        // Touch-friendly number bar shown only on mobile (via CSS).
        html! {
            <div class="number-bar">
                { for (1..=9).map(|n| {
                    let onclick = ctx.link().callback(move |_| Msg::PlaceNumber(n));
                    html! { <button onclick={onclick}>{ n }</button> }
                })}
                <button class="clear-btn"
                        onclick={ctx.link().callback(|_| Msg::PlaceNumber(0))}>
                    {"Clear"}
                </button>
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
                        let avg = if s.won > 0 { format_time(s.total_time / s.won) } else { "--:--".to_string() };
                        let label = match *d {
                            "easy" => "Easy", "medium" => "Medium", "hard" => "Hard", _ => d,
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
