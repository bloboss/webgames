use yew::prelude::*;
use gloo_timers::callback::Interval;

use crate::game::{Bottle, GameState};
use crate::generator::{self, Difficulty};
use crate::solver;
use crate::storage::{self, format_time};

pub enum Msg {
    NewGame,
    Reset,
    Undo,
    Hint,
    ClickTube(usize),
    SetDifficulty(&'static str),
    Tick,
    KeyDown(KeyboardEvent),
    ToggleStats,
    ToggleHistory,
}

pub struct App {
    initial: GameState,
    state: GameState,
    history: Vec<GameState>,
    difficulty: &'static str,
    selected: Option<usize>,
    hint_src: Option<usize>,
    hint_dst: Option<usize>,
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
        let state = generator::generate(diff);
        self.initial = state.clone();
        self.state = state;
        self.history.clear();
        self.selected = None;
        self.hint_src = None;
        self.hint_dst = None;
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
        if self.state.is_solved() {
            self.stop_timer();
            storage::record_win(self.difficulty, self.elapsed, self.state.moves);
            self.message = format!(
                "Solved in {} moves! Time: {}",
                self.state.moves,
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
            initial: GameState::new(vec![], 0),
            state: GameState::new(vec![], 0),
            history: Vec::new(),
            difficulty: "easy",
            selected: None,
            hint_src: None,
            hint_dst: None,
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
                    storage::record_abandon(self.difficulty, self.elapsed, self.state.moves);
                }
                self.start_game(ctx);
                true
            }
            Msg::Reset => {
                self.state = self.initial.clone();
                self.history.clear();
                self.selected = None;
                self.hint_src = None;
                self.hint_dst = None;
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
            Msg::Undo => {
                if let Some(prev) = self.history.pop() {
                    self.state = prev;
                    self.selected = None;
                    self.hint_src = None;
                    self.hint_dst = None;
                    self.message.clear();
                    self.message_is_win = false;
                    true
                } else {
                    self.message = "Nothing to undo.".to_string();
                    true
                }
            }
            Msg::Hint => {
                if !self.active {
                    return false;
                }
                match solver::solve(&self.state) {
                    Some(moves) if !moves.is_empty() => {
                        let m = &moves[0];
                        self.hint_src = Some(m.src);
                        self.hint_dst = Some(m.dst);
                        self.selected = None;
                        self.message = format!(
                            "Hint: pour tube {} -> tube {}",
                            m.src + 1,
                            m.dst + 1
                        );
                        self.message_is_win = false;
                        true
                    }
                    _ => {
                        self.message = "No solution found from current state.".to_string();
                        true
                    }
                }
            }
            Msg::ClickTube(idx) => {
                if !self.active {
                    return false;
                }
                self.hint_src = None;
                self.hint_dst = None;

                if let Some(src) = self.selected {
                    if src == idx {
                        // Deselect
                        self.selected = None;
                        return true;
                    }
                    // Try pour
                    if self.state.can_pour(src, idx) {
                        self.history.push(self.state.clone());
                        self.state.pour(src, idx);
                        self.selected = None;
                        self.message.clear();
                        self.message_is_win = false;
                        self.check_win();
                    } else {
                        self.message = "Invalid pour.".to_string();
                        self.message_is_win = false;
                        // If clicked a non-empty tube, select it instead
                        if !self.state.bottles[idx].is_empty() {
                            self.selected = Some(idx);
                        } else {
                            self.selected = None;
                        }
                    }
                    true
                } else {
                    // Select source (only if non-empty)
                    if idx < self.state.bottles.len() && !self.state.bottles[idx].is_empty() {
                        self.selected = Some(idx);
                        self.message.clear();
                        true
                    } else {
                        false
                    }
                }
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
                let key = e.key();
                if key == "Escape" {
                    self.selected = None;
                    self.hint_src = None;
                    self.hint_dst = None;
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
                <h1>{"W A T E R   S O R T"}</h1>

                { self.view_controls(ctx) }
                { self.view_info_bar() }
                { self.view_tubes(ctx) }

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
        let diffs: Vec<(&str, &str)> = vec![("easy", "Easy"), ("medium", "Medium"), ("hard", "Hard")];

        html! {
            <div class="controls">
                { for diffs.iter().map(|(val, label)| {
                    let v = *val;
                    let active = self.difficulty == v;
                    let onclick = ctx.link().callback(move |_| Msg::SetDifficulty(
                        match v { "easy" => "easy", "hard" => "hard", _ => "medium" }
                    ));
                    html! {
                        <button class={classes!(active.then_some("active"))} onclick={onclick}>
                            { label }
                        </button>
                    }
                })}
                <button onclick={ctx.link().callback(|_| Msg::NewGame)}>{"New Game"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Reset)}>{"Reset"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Undo)}>{"Undo"}</button>
                <button onclick={ctx.link().callback(|_| Msg::Hint)}>{"Hint"}</button>
            </div>
        }
    }

    fn view_info_bar(&self) -> Html {
        html! {
            <div class="info-bar">
                <div class="info-item">
                    <div class="label">{"Moves"}</div>
                    <div class="val">{ self.state.moves }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Time"}</div>
                    <div class="val">{ format_time(self.elapsed) }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Colors"}</div>
                    <div class="val">{ self.state.num_colors }</div>
                </div>
            </div>
        }
    }

    fn view_tubes(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="tubes">
                { for self.state.bottles.iter().enumerate().map(|(idx, bottle)| {
                    self.view_tube(ctx, idx, bottle)
                })}
            </div>
        }
    }

    fn view_tube(&self, ctx: &Context<Self>, idx: usize, bottle: &Bottle) -> Html {
        let is_selected = self.selected == Some(idx);
        let is_hint_src = self.hint_src == Some(idx);
        let is_hint_dst = self.hint_dst == Some(idx);
        let is_complete = bottle.is_complete();

        let mut classes = vec!["tube"];
        if is_selected {
            classes.push("selected");
        }
        if is_hint_src {
            classes.push("hint-src");
        }
        if is_hint_dst {
            classes.push("hint-dst");
        }
        if is_complete {
            classes.push("complete");
        }

        let onclick = ctx.link().callback(move |_| Msg::ClickTube(idx));

        html! {
            <div class={classes.join(" ")} onclick={onclick}>
                { for bottle.units.iter().map(|&color| {
                    let cls = format!("cube c{}", color);
                    html! { <div class={cls}></div> }
                })}
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
                        <th>{"Best Moves"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for diffs.iter().map(|d| {
                        let s = stats.get(d);
                        let bt = s.best_time.map(format_time).unwrap_or_else(|| "--:--".to_string());
                        let bm = s.best_moves.map(|m| m.to_string()).unwrap_or_else(|| "--".to_string());
                        let label = match *d {
                            "easy" => "Easy", "medium" => "Medium", "hard" => "Hard", _ => d,
                        };
                        html! {
                            <tr>
                                <td>{ label }</td>
                                <td>{ s.played }</td>
                                <td>{ s.won }</td>
                                <td>{ bt }</td>
                                <td>{ bm }</td>
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
                        <th>{"Moves"}</th>
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
                                <td>{ h.moves }</td>
                                <td>{ format_time(h.time_secs) }</td>
                            </tr>
                        }
                    })}
                </tbody>
            </table>
        }
    }
}
