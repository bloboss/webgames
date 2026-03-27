use yew::prelude::*;

use crate::config::PuzzleConfig;
use crate::config_page::ConfigPage;
use crate::play_page::PlayPage;

pub enum Msg {
    StartGame(PuzzleConfig),
    BackToConfig,
}

enum Page {
    Config,
    Play(PuzzleConfig),
}

pub struct App {
    page: Page,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        App {
            page: Page::Config,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StartGame(config) => {
                self.page = Page::Play(config);
                true
            }
            Msg::BackToConfig => {
                self.page = Page::Config;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="app">
                <h1>{"G E N E R I C   M U L T I - S U D O K U"}</h1>
                { match &self.page {
                    Page::Config => {
                        let on_start = ctx.link().callback(Msg::StartGame);
                        html! { <ConfigPage on_start_game={on_start} /> }
                    }
                    Page::Play(config) => {
                        let on_back = ctx.link().callback(|_| Msg::BackToConfig);
                        html! { <PlayPage config={config.clone()} on_back={on_back} /> }
                    }
                }}
            </div>
        }
    }
}
