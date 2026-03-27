use yew::prelude::*;

use crate::config::{self, BoardPlacement, PuzzleConfig};
use crate::storage;

/// Maximum meta-grid dimensions for the configuration canvas.
const META_GRID_SIZE: usize = 9;

/// Board colors for up to 8 boards.
const BOARD_COLORS: &[&str] = &[
    "#ee5a24", "#00d2d3", "#feca57", "#54a0ff",
    "#a29bfe", "#fd79a8", "#00b894", "#e17055",
];

pub enum Msg {
    PlaceBoard(usize, usize),
    RemoveBoard(usize),
    ClearAll,
    SetName(String),
    SaveConfig,
    LoadPreset(usize),
    LoadSaved(usize),
    DeleteSaved(usize),
    StartGame,
    ImportConfig,
    SetImportText(String),
    ExportConfig,
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_start_game: Callback<PuzzleConfig>,
}

pub struct ConfigPage {
    config: PuzzleConfig,
    message: String,
    import_text: String,
    export_text: String,
}

impl Component for ConfigPage {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        ConfigPage {
            config: PuzzleConfig {
                name: "Custom".to_string(),
                boards: vec![BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                }],
            },
            message: String::new(),
            import_text: String::new(),
            export_text: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PlaceBoard(mr, mc) => {
                // Check if clicking would exceed grid bounds
                if mr + 3 > META_GRID_SIZE || mc + 3 > META_GRID_SIZE {
                    self.message = "Board would exceed grid bounds.".to_string();
                    return true;
                }
                // Check for existing board at same position
                if self
                    .config
                    .boards
                    .iter()
                    .any(|b| b.meta_row == mr && b.meta_col == mc)
                {
                    self.message = "A board already exists at this position.".to_string();
                    return true;
                }
                // Tentatively add and validate
                self.config.boards.push(BoardPlacement {
                    meta_row: mr,
                    meta_col: mc,
                });
                if let Err(e) = self.config.validate() {
                    self.config.boards.pop();
                    self.message = e;
                } else {
                    self.message = format!(
                        "Board {} placed at ({}, {}).",
                        self.config.boards.len(),
                        mr,
                        mc
                    );
                }
                true
            }
            Msg::RemoveBoard(idx) => {
                if self.config.boards.len() <= 1 {
                    self.message = "Must keep at least one board.".to_string();
                } else if idx < self.config.boards.len() {
                    self.config.boards.remove(idx);
                    self.message = format!("Board {} removed.", idx + 1);
                }
                true
            }
            Msg::ClearAll => {
                self.config.boards.clear();
                self.config.boards.push(BoardPlacement {
                    meta_row: 0,
                    meta_col: 0,
                });
                self.message = "Reset to single board.".to_string();
                true
            }
            Msg::SetName(name) => {
                self.config.name = name;
                false
            }
            Msg::SaveConfig => {
                if self.config.name.is_empty() {
                    self.message = "Please enter a name.".to_string();
                } else {
                    storage::save_config(&self.config);
                    self.message = format!("Config '{}' saved.", self.config.name);
                }
                true
            }
            Msg::LoadPreset(idx) => {
                let presets = config::presets();
                if idx < presets.len() {
                    self.config = presets[idx].clone();
                    self.message = format!("Loaded preset: {}", self.config.name);
                }
                true
            }
            Msg::LoadSaved(idx) => {
                let saved = storage::load_saved_configs();
                if idx < saved.len() {
                    self.config = saved[idx].clone();
                    self.message = format!("Loaded config: {}", self.config.name);
                }
                true
            }
            Msg::DeleteSaved(idx) => {
                let saved = storage::load_saved_configs();
                if idx < saved.len() {
                    let name = saved[idx].name.clone();
                    storage::delete_config(&name);
                    self.message = format!("Deleted config: {}", name);
                }
                true
            }
            Msg::StartGame => {
                if let Err(e) = self.config.validate() {
                    self.message = e;
                    return true;
                }
                ctx.props().on_start_game.emit(self.config.clone());
                false
            }
            Msg::ImportConfig => {
                if let Some(cfg) = PuzzleConfig::from_json(&self.import_text) {
                    if let Err(e) = cfg.validate() {
                        self.message = format!("Invalid config: {}", e);
                    } else {
                        self.config = cfg;
                        self.message = "Config imported.".to_string();
                    }
                } else {
                    self.message = "Failed to parse JSON.".to_string();
                }
                true
            }
            Msg::SetImportText(text) => {
                self.import_text = text;
                false
            }
            Msg::ExportConfig => {
                self.export_text = self.config.to_json();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="config-page">
                <h2>{"Configure Layout"}</h2>

                <div class="config-sections">
                    { self.view_meta_grid(ctx) }
                    { self.view_sidebar(ctx) }
                </div>

                <div class="message">{ &self.message }</div>

                <div class="config-actions">
                    <button class="primary-btn" onclick={ctx.link().callback(|_| Msg::StartGame)}>
                        {"Play This Layout"}
                    </button>
                </div>

                { self.view_import_export(ctx) }
            </div>
        }
    }
}

impl ConfigPage {
    fn view_meta_grid(&self, ctx: &Context<Self>) -> Html {
        let overlap_info = self.config.compute_overlaps();

        html! {
            <div class="meta-grid-section">
                <h3>{"Click to place boards (each board = 3x3 region)"}</h3>
                <div class="meta-grid" style={format!(
                    "grid-template-columns: repeat({}, 44px); grid-template-rows: repeat({}, 44px);",
                    META_GRID_SIZE, META_GRID_SIZE
                )}>
                    { for (0..META_GRID_SIZE).flat_map(|mr| {
                        (0..META_GRID_SIZE).map(move |mc| (mr, mc))
                    }).map(|(mr, mc)| {
                        let owners = overlap_info.owners_of_meta(mr, mc);
                        let onclick = ctx.link().callback(move |_| Msg::PlaceBoard(mr, mc));

                        let (bg, label) = if owners.len() > 1 {
                            // Overlap: blend colors
                            ("rgba(255, 255, 255, 0.15)".to_string(),
                             owners.iter().map(|o| format!("{}", o + 1)).collect::<Vec<_>>().join("+"))
                        } else if owners.len() == 1 {
                            let idx = owners[0] % BOARD_COLORS.len();
                            (format!("{}33", BOARD_COLORS[idx]), // 20% opacity hex
                             format!("{}", owners[0] + 1))
                        } else {
                            (String::new(), String::new())
                        };

                        let border_color = if !owners.is_empty() {
                            let idx = owners[0] % BOARD_COLORS.len();
                            BOARD_COLORS[idx].to_string()
                        } else {
                            "#2d3436".to_string()
                        };

                        html! {
                            <div class={classes!("meta-cell", (!owners.is_empty()).then_some("occupied"))}
                                 style={format!("background: {}; border-color: {};", bg, border_color)}
                                 onclick={onclick}>
                                { label }
                            </div>
                        }
                    })}
                </div>
                <div class="board-legend">
                    { for self.config.boards.iter().enumerate().map(|(i, b)| {
                        let idx = i % BOARD_COLORS.len();
                        let remove = ctx.link().callback(move |_| Msg::RemoveBoard(i));
                        html! {
                            <div class="legend-item">
                                <span class="legend-color" style={format!("background: {};", BOARD_COLORS[idx])}></span>
                                <span>{ format!("Board {} ({},{})", i + 1, b.meta_row, b.meta_col) }</span>
                                <button class="remove-btn" onclick={remove}>{"x"}</button>
                            </div>
                        }
                    })}
                </div>
                <button onclick={ctx.link().callback(|_| Msg::ClearAll)}>{"Clear All"}</button>
            </div>
        }
    }

    fn view_sidebar(&self, ctx: &Context<Self>) -> Html {
        let presets = config::presets();
        let saved = storage::load_saved_configs();

        html! {
            <div class="config-sidebar">
                <div class="config-name">
                    <label>{"Config Name:"}</label>
                    <input type="text"
                           value={self.config.name.clone()}
                           onchange={ctx.link().callback(|e: Event| {
                               let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                               Msg::SetName(input.value())
                           })} />
                    <button onclick={ctx.link().callback(|_| Msg::SaveConfig)}>{"Save"}</button>
                </div>

                <div class="preset-list">
                    <h3>{"Presets"}</h3>
                    { for presets.iter().enumerate().map(|(i, p)| {
                        let onclick = ctx.link().callback(move |_| Msg::LoadPreset(i));
                        html! {
                            <button class="preset-btn" onclick={onclick}>
                                { &p.name }
                                <span class="board-count">{ format!(" ({})", p.boards.len()) }</span>
                            </button>
                        }
                    })}
                </div>

                { if !saved.is_empty() {
                    html! {
                        <div class="saved-list">
                            <h3>{"Saved Configs"}</h3>
                            { for saved.iter().enumerate().map(|(i, s)| {
                                let load = ctx.link().callback(move |_| Msg::LoadSaved(i));
                                let delete = ctx.link().callback(move |_| Msg::DeleteSaved(i));
                                html! {
                                    <div class="saved-item">
                                        <button class="preset-btn" onclick={load}>
                                            { &s.name }
                                            <span class="board-count">{ format!(" ({})", s.boards.len()) }</span>
                                        </button>
                                        <button class="remove-btn" onclick={delete}>{"x"}</button>
                                    </div>
                                }
                            })}
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        }
    }

    fn view_import_export(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="import-export">
                <h3>{"Import / Export"}</h3>
                <div class="ie-row">
                    <textarea
                        placeholder="Paste JSON config here..."
                        value={self.import_text.clone()}
                        onchange={ctx.link().callback(|e: Event| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            Msg::SetImportText(input.value())
                        })}
                    />
                    <button onclick={ctx.link().callback(|_| Msg::ImportConfig)}>{"Import"}</button>
                    <button onclick={ctx.link().callback(|_| Msg::ExportConfig)}>{"Export"}</button>
                </div>
                { if !self.export_text.is_empty() {
                    html! {
                        <pre class="export-output">{ &self.export_text }</pre>
                    }
                } else {
                    html! {}
                }}
            </div>
        }
    }
}
