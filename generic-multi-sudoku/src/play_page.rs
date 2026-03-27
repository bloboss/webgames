use yew::prelude::*;
use gloo_timers::callback::Interval;

use crate::config::PuzzleConfig;
use crate::generator::{self, Difficulty};
use crate::generic_board::GenericMultiBoard;
use crate::storage::{self, format_time};

/// Board colors matching config page.
const BOARD_COLORS: &[&str] = &[
    "#ee5a24", "#00d2d3", "#feca57", "#54a0ff",
    "#a29bfe", "#fd79a8", "#00b894", "#e17055",
];

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
    BackToConfig,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub config: PuzzleConfig,
    pub on_back: Callback<()>,
}

pub struct PlayPage {
    puzzle: GenericMultiBoard,
    solution: GenericMultiBoard,
    current: GenericMultiBoard,
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

impl PlayPage {
    fn start_game(&mut self, ctx: &Context<Self>) {
        let diff = match self.difficulty {
            "easy" => Difficulty::Easy,
            "hard" => Difficulty::Hard,
            _ => Difficulty::Medium,
        };
        let (puzzle, solution) = generator::generate(&ctx.props().config, diff);
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

    fn check_win(&mut self, config: &PuzzleConfig) {
        if self.current.is_solved() {
            self.stop_timer();
            storage::record_win(config, self.difficulty, self.elapsed);
            self.message = format!("All boards complete! Time: {}", format_time(self.elapsed));
            self.message_is_win = true;
        }
    }

    fn next_active(
        board: &GenericMultiBoard,
        gr: usize,
        gc: usize,
        dr: i32,
        dc: i32,
    ) -> Option<(usize, usize)> {
        let rows = board.grid_rows() as i32;
        let cols = board.grid_cols() as i32;
        let mut r = gr as i32 + dr;
        let mut c = gc as i32 + dc;
        while r >= 0 && r < rows && c >= 0 && c < cols {
            if board.is_active(r as usize, c as usize) {
                return Some((r as usize, c as usize));
            }
            r += dr;
            c += dc;
        }
        None
    }
}

impl Component for PlayPage {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let config = &ctx.props().config;
        let dummy = GenericMultiBoard::new(config);
        let mut page = PlayPage {
            puzzle: dummy.clone(),
            solution: dummy.clone(),
            current: dummy,
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
        page.start_game(ctx);
        page
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let config = ctx.props().config.clone();
        match msg {
            Msg::NewGame => {
                if self.active {
                    storage::record_abandon(&config, self.difficulty, self.elapsed);
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
                let rows = self.current.grid_rows();
                let cols = self.current.grid_cols();
                for gr in 0..rows {
                    for gc in 0..cols {
                        if self.current.is_active(gr, gc) && self.current.is_empty(gr, gc) {
                            let val = self.solution.get(gr, gc);
                            self.current.set(gr, gc, val);
                            self.selected = Some((gr, gc));
                            self.message = format!("Hint: {} placed", val);
                            self.message_is_win = false;
                            self.check_win(&config);
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
                let rows = self.current.grid_rows();
                let cols = self.current.grid_cols();
                for gr in 0..rows {
                    for gc in 0..cols {
                        if self.current.is_active(gr, gc) && self.puzzle.is_empty(gr, gc) {
                            self.current.set(gr, gc, self.solution.get(gr, gc));
                        }
                    }
                }
                self.message = "Solved. Start a new game to play again.".to_string();
                self.message_is_win = false;
                true
            }
            Msg::SelectCell(gr, gc) => {
                if self.active && self.current.is_active(gr, gc) {
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
                    if self.current.is_active(gr, gc) && self.puzzle.is_empty(gr, gc) {
                        self.current.set(gr, gc, num);
                        self.message.clear();
                        self.message_is_win = false;
                        if num != 0 {
                            self.check_win(&config);
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
                            if let Some(next) = Self::next_active(&self.current, gr, gc, -1, 0) {
                                self.selected = Some(next);
                                return true;
                            }
                        }
                        false
                    }
                    "ArrowDown" => {
                        if let Some((gr, gc)) = self.selected {
                            if let Some(next) = Self::next_active(&self.current, gr, gc, 1, 0) {
                                self.selected = Some(next);
                                return true;
                            }
                        }
                        false
                    }
                    "ArrowLeft" => {
                        if let Some((gr, gc)) = self.selected {
                            if let Some(next) = Self::next_active(&self.current, gr, gc, 0, -1) {
                                self.selected = Some(next);
                                return true;
                            }
                        }
                        false
                    }
                    "ArrowRight" => {
                        if let Some((gr, gc)) = self.selected {
                            if let Some(next) = Self::next_active(&self.current, gr, gc, 0, 1) {
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
            Msg::BackToConfig => {
                if self.active {
                    storage::record_abandon(&config, self.difficulty, self.elapsed);
                }
                ctx.props().on_back.emit(());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let config = &ctx.props().config;
        let on_keydown = ctx.link().callback(Msg::KeyDown);

        html! {
            <div class="play-page" onkeydown={on_keydown} tabindex="0">
                <div class="play-header">
                    <button onclick={ctx.link().callback(|_| Msg::BackToConfig)}>
                        {"<- Configure"}
                    </button>
                    <h2>{ format!("{} ({} boards)", config.name, config.boards.len()) }</h2>
                </div>

                { self.view_controls(ctx) }

                <div class="game-area">
                    { self.view_grid(ctx) }
                    { self.view_sidebar(ctx) }
                </div>

                <div class={classes!("message", self.message_is_win.then_some("win"))}>
                    { &self.message }
                </div>

                { self.view_stats_section(ctx) }
            </div>
        }
    }
}

impl PlayPage {
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
        let rows = self.current.grid_rows();
        let cols = self.current.grid_cols();
        let config = &ctx.props().config;

        // Compute cell size based on grid dimensions
        let cell_size = if cols > 21 || rows > 21 { 28 } else { 36 };

        html! {
            <div class="multi-grid" style={format!(
                "grid-template-columns: repeat({}, {}px); grid-template-rows: repeat({}, {}px);",
                cols, cell_size, rows, cell_size
            )}>
                { for (0..rows).flat_map(|gr| {
                    (0..cols).map(move |gc| (gr, gc))
                }).map(|(gr, gc)| {
                    self.view_cell(ctx, config, gr, gc, cell_size)
                })}
            </div>
        }
    }

    fn view_cell(
        &self,
        ctx: &Context<Self>,
        config: &PuzzleConfig,
        gr: usize,
        gc: usize,
        cell_size: usize,
    ) -> Html {
        if !self.current.is_active(gr, gc) {
            return html! {
                <div class="cell-spacer" style={format!(
                    "width: {}px; height: {}px;", cell_size, cell_size
                )}></div>
            };
        }

        let pv = self.puzzle.get(gr, gc);
        let cv = self.current.get(gr, gc);
        let sv = self.solution.get(gr, gc);
        let is_overlap = self.current.is_overlap(gr, gc);

        let is_selected = self.selected == Some((gr, gc));
        let is_highlighted = if let Some((sr, sc)) = self.selected {
            let sel_owners = self.current.owning_boards(sr, sc);
            let cell_owners = self.current.owning_boards(gr, gc);
            sel_owners.iter().any(|&sb| {
                cell_owners.iter().any(|&cb| {
                    if sb != cb {
                        return false;
                    }
                    let (slr, slc) = config.grid_to_local(sb, sr, sc);
                    let (clr, clc) = config.grid_to_local(cb, gr, gc);
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

        // Determine borders: thick at board edges and 3x3 box boundaries
        let owners = self.current.owning_boards(gr, gc);
        for &b in &owners {
            let (lr, lc) = config.grid_to_local(b, gr, gc);
            if lc % 3 == 2 && lc != 8 {
                classes.push("border-right");
            }
            if lr % 3 == 2 && lr != 8 {
                classes.push("border-bottom");
            }
            if lc == 8 {
                classes.push("border-right");
            }
            if lr == 8 {
                classes.push("border-bottom");
            }
            if lc == 0 {
                classes.push("border-left-thick");
            }
            if lr == 0 {
                classes.push("border-top-thick");
            }
        }

        // Tint based on primary owning board
        let tint = if !owners.is_empty() {
            let idx = owners[0] % BOARD_COLORS.len();
            format!("border-color: {}44;", BOARD_COLORS[idx])
        } else {
            String::new()
        };

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
        let font_size = if cell_size < 32 { 14 } else { 16 };

        html! {
            <div class={classes.join(" ")}
                 style={format!("{}font-size: {}px;", tint, font_size)}
                 onclick={onclick}>
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

    fn view_stats_section(&self, ctx: &Context<Self>) -> Html {
        let config = &ctx.props().config;

        html! {
            <div class="stats-section">
                <button onclick={ctx.link().callback(|_| Msg::ToggleStats)}>
                    { if self.show_stats { "Hide Stats" } else { "Show Stats" } }
                </button>
                <button onclick={ctx.link().callback(|_| Msg::ToggleHistory)}>
                    { if self.show_history { "Hide History" } else { "Show History" } }
                </button>

                { if self.show_stats { self.view_stats(config) } else { html! {} } }
                { if self.show_history { self.view_history(config) } else { html! {} } }
            </div>
        }
    }

    fn view_stats(&self, config: &PuzzleConfig) -> Html {
        let stats = storage::load_stats(config);
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

    fn view_history(&self, config: &PuzzleConfig) -> Html {
        let history = storage::load_history(config);
        if history.is_empty() {
            return html! { <p>{"No games played yet for this config."}</p> };
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
