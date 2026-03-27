mod board;
mod generator;
mod multi_board;
mod solver;

mod app;
mod storage;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<app::App>::new().render();
}
