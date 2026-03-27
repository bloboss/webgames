mod board;
mod config;
mod generator;
mod generic_board;
mod solver;

mod app;
mod config_page;
mod play_page;
mod storage;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<app::App>::new().render();
}
