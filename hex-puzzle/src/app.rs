use yew::prelude::*;
use gloo_timers::callback::{Interval, Timeout};

use crate::game::{Cell, Direction, GameState, Hex, Move, Rng};
use crate::storage::{self, format_time};

const HEX_SIZE: f64 = 36.0;
const SQRT3: f64 = 1.7320508075688772;
const EXIT_ANIM_MS: u32 = 280;
const BUMP_ANIM_MS: u32 = 280;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnimKind {
    Exit,
    Bump,
}

#[derive(Clone, Copy, Debug)]
struct Anim {
    hex: Hex,
    dx: f64,
    dy: f64,
    kind: AnimKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
    Master,
}

impl Difficulty {
    fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Expert => "Expert",
            Difficulty::Master => "Master",
        }
    }

    fn radius(self) -> i32 {
        match self {
            Difficulty::Easy => 2,
            Difficulty::Medium => 3,
            Difficulty::Hard => 3,
            Difficulty::Expert => 4,
            Difficulty::Master => 4,
        }
    }

    fn tiles(self) -> u32 {
        match self {
            Difficulty::Easy => 8,
            Difficulty::Medium => 14,
            Difficulty::Hard => 22,
            Difficulty::Expert => 32,
            Difficulty::Master => 50,
        }
    }
}

pub enum Msg {
    NewGame,
    SetDifficulty(Difficulty),
    HexClick(Hex),
    CommitMove(Hex),
    TimerTick,
    ToggleStats,
    ToggleHistory,
}

pub struct App {
    game: GameState,
    rng: Rng,
    difficulty: Difficulty,
    elapsed: u32,
    message: String,
    message_is_good: bool,
    last_clicked: Option<Hex>,
    show_stats: bool,
    show_history: bool,
    recorded: bool,
    animating: Option<Anim>,
    _clock: Option<Interval>,
    _anim_timeout: Option<Timeout>,
}

impl App {
    fn start_game(&mut self, ctx: &Context<Self>) {
        let radius = self.difficulty.radius();
        let tiles = self.difficulty.tiles();
        let (game, _order) = GameState::generate(radius, tiles, &mut self.rng);
        self.game = game;
        self.elapsed = 0;
        self.message.clear();
        self.message_is_good = false;
        self.last_clicked = None;
        self.recorded = false;
        self.animating = None;
        self._anim_timeout = None;
        self.start_timer(ctx);
        storage::save_game(&self.game, self.elapsed, self.difficulty.label());
    }

    fn start_timer(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        self._clock = Some(Interval::new(1000, move || {
            link.send_message(Msg::TimerTick);
        }));
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut app = App {
            game: GameState::empty(3),
            rng: Rng::from_entropy(),
            difficulty: Difficulty::Medium,
            elapsed: 0,
            message: String::new(),
            message_is_good: false,
            last_clicked: None,
            show_stats: false,
            show_history: false,
            recorded: false,
            animating: None,
            _clock: None,
            _anim_timeout: None,
        };

        if let Some(save) = storage::load_game() {
            app.game = save.game;
            app.elapsed = save.elapsed;
            app.difficulty = match save.difficulty.as_str() {
                "Easy" => Difficulty::Easy,
                "Hard" => Difficulty::Hard,
                "Expert" => Difficulty::Expert,
                "Master" => Difficulty::Master,
                _ => Difficulty::Medium,
            };
            app.start_timer(ctx);
        } else {
            app.start_game(ctx);
        }

        app
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NewGame => {
                if !self.recorded && self.game.moves > 0 && !self.game.is_won() {
                    storage::record_game(
                        self.difficulty.label(),
                        self.game.moves,
                        self.elapsed,
                        false,
                    );
                }
                storage::clear_save();
                self.start_game(ctx);
                true
            }
            Msg::SetDifficulty(d) => {
                if self.difficulty != d {
                    self.difficulty = d;
                    storage::clear_save();
                    self.start_game(ctx);
                }
                true
            }
            Msg::HexClick(hex) => {
                if self.game.is_won() || self.animating.is_some() {
                    return false;
                }
                let dir = match self.game.get(hex) {
                    Some(Cell::Tile(d)) => d,
                    _ => {
                        self.message.clear();
                        self.last_clicked = None;
                        return true;
                    }
                };
                self.last_clicked = Some(hex);
                let (step_dx, step_dy) = dir_pixel_delta(dir);
                let steps = self.game.empty_steps(hex, dir);
                let path_clear = self.game.path_clear(hex, dir);
                let (kind, dist) = if path_clear {
                    (AnimKind::Exit, (steps + 1) as f64)
                } else {
                    (AnimKind::Bump, (steps as f64).max(0.4))
                };
                self.animating = Some(Anim {
                    hex,
                    dx: step_dx * dist,
                    dy: step_dy * dist,
                    kind,
                });
                let link = ctx.link().clone();
                let delay = match kind {
                    AnimKind::Exit => EXIT_ANIM_MS,
                    AnimKind::Bump => BUMP_ANIM_MS,
                };
                self._anim_timeout = Some(Timeout::new(delay, move || {
                    link.send_message(Msg::CommitMove(hex));
                }));
                true
            }
            Msg::CommitMove(hex) => {
                self.animating = None;
                self._anim_timeout = None;
                match self.game.play(hex) {
                    Move::Removed => {
                        if self.game.is_won() {
                            self.message = format!(
                                "Solved in {} moves!",
                                self.game.moves
                            );
                            self.message_is_good = true;
                            if !self.recorded {
                                storage::record_game(
                                    self.difficulty.label(),
                                    self.game.moves,
                                    self.elapsed,
                                    true,
                                );
                                self.recorded = true;
                            }
                            storage::clear_save();
                            self._clock = None;
                        } else if self.game.is_stuck() {
                            self.message = "Stuck — no tile can move.".to_string();
                            self.message_is_good = false;
                            storage::save_game(
                                &self.game,
                                self.elapsed,
                                self.difficulty.label(),
                            );
                        } else {
                            self.message.clear();
                            storage::save_game(
                                &self.game,
                                self.elapsed,
                                self.difficulty.label(),
                            );
                        }
                    }
                    Move::Blocked => {
                        self.message = "Blocked.".to_string();
                        self.message_is_good = false;
                    }
                    Move::Empty => {
                        self.message.clear();
                        self.last_clicked = None;
                    }
                }
                true
            }
            Msg::TimerTick => {
                if !self.game.is_won() {
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
                <h1>{"H E X"}</h1>
                { self.view_controls(ctx) }
                { self.view_info_bar() }
                { self.view_board(ctx) }

                <div class={classes!("message", self.message_is_good.then_some("good"))}>
                    { &self.message }
                </div>

                <div class="rules">
                    <p>{"Click a hex to launch it in the direction of its arrow."}</p>
                    <p>{"If the path is clear to the edge, it falls off the board. Otherwise it snaps back."}</p>
                    <p>{"Remove every hex to win."}</p>
                </div>

                { self.view_stats_section(ctx) }
            </div>
        }
    }
}

impl App {
    fn view_controls(&self, ctx: &Context<Self>) -> Html {
        let mk = |d: Difficulty| {
            let cls = if self.difficulty == d { "diff active" } else { "diff" };
            html! {
                <button class={cls}
                    onclick={ctx.link().callback(move |_| Msg::SetDifficulty(d))}>
                    { d.label() }
                </button>
            }
        };
        html! {
            <div class="controls">
                <button class="primary"
                    onclick={ctx.link().callback(|_| Msg::NewGame)}>
                    {"New Puzzle"}
                </button>
                <div class="diff-group">
                    { mk(Difficulty::Easy) }
                    { mk(Difficulty::Medium) }
                    { mk(Difficulty::Hard) }
                    { mk(Difficulty::Expert) }
                    { mk(Difficulty::Master) }
                </div>
            </div>
        }
    }

    fn view_info_bar(&self) -> Html {
        html! {
            <div class="info-bar">
                <div class="info-item">
                    <div class="label">{"Tiles"}</div>
                    <div class="val">{ self.game.tile_count() }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Removed"}</div>
                    <div class="val">{ format!("{} / {}", self.game.removed, self.game.total) }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Moves"}</div>
                    <div class="val">{ self.game.moves }</div>
                </div>
                <div class="info-item">
                    <div class="label">{"Time"}</div>
                    <div class="val">{ format_time(self.elapsed) }</div>
                </div>
            </div>
        }
    }

    fn view_board(&self, ctx: &Context<Self>) -> Html {
        let r = self.game.radius as f64;
        let pad = HEX_SIZE * 0.6;
        let width = SQRT3 * HEX_SIZE * (2.0 * r + 1.0) + pad * 2.0;
        let height = HEX_SIZE * (1.5 * (2.0 * r) + 2.0) + pad * 2.0;
        let cx = width / 2.0;
        let cy = height / 2.0;
        let viewbox = format!("0 0 {} {}", width, height);

        let cells: Vec<Html> = self
            .game
            .cells
            .iter()
            .map(|(hex, cell)| self.view_hex(ctx, *hex, *cell, cx, cy))
            .collect();

        html! {
            <div class="board-wrap">
                <svg class="board" viewBox={viewbox}
                    preserveAspectRatio="xMidYMid meet">
                    { for cells }
                </svg>
            </div>
        }
    }

    fn view_hex(&self, ctx: &Context<Self>, hex: Hex, cell: Cell, cx: f64, cy: f64) -> Html {
        let (x, y) = hex_to_pixel(hex, cx, cy);
        let points = hex_corners(x, y);
        let onclick = ctx.link().callback(move |_| Msg::HexClick(hex));

        match cell {
            Cell::Empty => {
                html! {
                    <g class="cell-group empty" onclick={onclick}>
                        <polygon class="cell empty" points={points} />
                    </g>
                }
            }
            Cell::Tile(dir) => {
                let tier = dir as usize;
                let mut cls = format!("cell-group tile t{}", tier);
                if self.last_clicked == Some(hex) {
                    cls.push_str(" recent");
                }
                let mut style: Option<String> = None;
                if let Some(anim) = self.animating {
                    if anim.hex == hex {
                        match anim.kind {
                            AnimKind::Exit => cls.push_str(" exiting"),
                            AnimKind::Bump => cls.push_str(" bumping"),
                        }
                        style = Some(format!(
                            "--anim-dx: {:.2}px; --anim-dy: {:.2}px;",
                            anim.dx, anim.dy
                        ));
                    }
                }
                let arrow = arrow_path(x, y, dir);
                html! {
                    <g class={cls} onclick={onclick} style={style}>
                        <polygon class="cell" points={points} />
                        <path class="arrow" d={arrow} />
                    </g>
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
        let best = match stats.best_time {
            Some(t) => format_time(t),
            None => "--".to_string(),
        };
        html! {
            <table class="stats-table">
                <tbody>
                    <tr><td>{"Puzzles Solved"}</td><td>{ stats.puzzles_solved }</td></tr>
                    <tr><td>{"Puzzles Attempted"}</td><td>{ stats.puzzles_attempted }</td></tr>
                    <tr><td>{"Total Moves"}</td><td>{ stats.total_moves }</td></tr>
                    <tr><td>{"Best Time"}</td><td>{ best }</td></tr>
                    <tr><td>{"Total Time"}</td><td>{ format_time(stats.total_time) }</td></tr>
                </tbody>
            </table>
        }
    }

    fn view_history(&self) -> Html {
        let history = storage::load_history();
        if history.is_empty() {
            return html! { <p class="empty-history">{"No games played yet."}</p> };
        }
        let recent: Vec<_> = history.iter().rev().take(20).collect();
        html! {
            <table class="history-table">
                <thead>
                    <tr>
                        <th>{"Date"}</th>
                        <th>{"Difficulty"}</th>
                        <th>{"Moves"}</th>
                        <th>{"Time"}</th>
                        <th>{"Result"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for recent.iter().map(|h| html! {
                        <tr>
                            <td>{ &h.date }</td>
                            <td>{ &h.difficulty }</td>
                            <td>{ h.moves }</td>
                            <td>{ format_time(h.time_secs) }</td>
                            <td>{ if h.solved { "Solved" } else { "—" } }</td>
                        </tr>
                    })}
                </tbody>
            </table>
        }
    }
}

/// Per-step pixel delta when sliding from one hex centre to the next
/// neighbour in the given direction (pointy-top axial layout).
fn dir_pixel_delta(dir: Direction) -> (f64, f64) {
    let (dq, dr) = dir.delta();
    let dx = HEX_SIZE * SQRT3 * (dq as f64 + dr as f64 / 2.0);
    let dy = HEX_SIZE * 1.5 * dr as f64;
    (dx, dy)
}

/// Pointy-top hex → pixel.
fn hex_to_pixel(hex: Hex, cx: f64, cy: f64) -> (f64, f64) {
    let q = hex.q as f64;
    let r = hex.r as f64;
    let x = cx + HEX_SIZE * SQRT3 * (q + r / 2.0);
    let y = cy + HEX_SIZE * 1.5 * r;
    (x, y)
}

/// SVG `points` string for a pointy-top hex centred at (x, y).
fn hex_corners(x: f64, y: f64) -> String {
    let mut s = String::new();
    for i in 0..6 {
        let angle = std::f64::consts::PI / 180.0 * (60.0 * i as f64 - 30.0);
        let px = x + HEX_SIZE * angle.cos();
        let py = y + HEX_SIZE * angle.sin();
        if i > 0 {
            s.push(' ');
        }
        s.push_str(&format!("{:.2},{:.2}", px, py));
    }
    s
}

/// SVG path string for the arrow inside a hex.
fn arrow_path(x: f64, y: f64, dir: Direction) -> String {
    let angle = dir.angle_deg() * std::f64::consts::PI / 180.0;
    let len = HEX_SIZE * 0.55;
    let head = HEX_SIZE * 0.22;
    let (ca, sa) = (angle.cos(), angle.sin());
    // Tail and tip in local space (along x-axis), then rotated.
    let tail = (-len / 2.0, 0.0);
    let tip = (len / 2.0, 0.0);
    let left = (tip.0 - head, -head * 0.7);
    let right = (tip.0 - head, head * 0.7);

    let rot = |p: (f64, f64)| {
        (
            x + p.0 * ca - p.1 * sa,
            y + p.0 * sa + p.1 * ca,
        )
    };
    let t = rot(tail);
    let h = rot(tip);
    let l = rot(left);
    let r = rot(right);

    format!(
        "M {:.2} {:.2} L {:.2} {:.2} M {:.2} {:.2} L {:.2} {:.2} L {:.2} {:.2}",
        t.0, t.1, h.0, h.1, l.0, l.1, h.0, h.1, r.0, r.1
    )
}
