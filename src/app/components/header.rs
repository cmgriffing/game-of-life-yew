#[allow(dead_code)]
use log::*;
use web_sys::{EventTarget, HtmlSelectElement, InputEvent};
use yew::prelude::*;
use yew_components::Select;

use game_of_life_core::core::game::{Cellule, GameState, LifeState};
use game_of_life_core::core::history::History;
use game_of_life_core::core::seeds::seeds::{seed_middle_line_starter, Seed};

#[derive(Clone, PartialEq)]
pub enum Msg {
    Reset,
    SeedChanged(usize),
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub inset: bool,
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub active_count: i32,
    #[prop_or_default]
    pub modification_count: i32,
    #[prop_or_default]
    pub step_count: i32,

    #[prop_or_default]
    pub on_reset: Callback<()>,
    #[prop_or_default]
    pub on_seed_change: Callback<Seed>,

    #[prop_or_default]
    pub seed_options: Vec<Seed>,
}

pub struct AppHeader {
    props: Props,
    link: ComponentLink<Self>,
    current_seed: Seed,
}

impl Component for AppHeader {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            current_seed: Seed {
                cellules: vec![],
                label: "Loading".to_owned(),
            },
        }
    }

    #[allow(dead_code)]
    fn update(&mut self, message: Self::Message) -> ShouldRender {
        match message {
            Msg::Reset => {
                // false
                info!("reset happening");
                self.props.on_seed_change.emit(self.current_seed.clone());
            }
            Msg::SeedChanged(seed_option_index) => {
                info!("Seed changed");
                let seed_option = self.props.seed_options[seed_option_index].clone();
                self.current_seed = seed_option.clone();
                self.props.on_seed_change.emit(seed_option);
            }
        }

        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props.clone();
        if self.current_seed.label == "Loading".to_owned() {
            self.current_seed = props.seed_options[0].clone();
            props.on_seed_change.emit(props.seed_options[0].clone());
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
                <h1>{ "Life of Game of Life" }</h1>
                <header class="main-header">
                    <div class="controls">
                        <select onchange=self.link.callback(|event: ChangeData| match event {
                            ChangeData::Select(element) => {
                                info!("element");
                                Msg::SeedChanged(element.selected_index() as usize)
                            }
                            _ => unimplemented!()
                        })>
                            {self.props.seed_options.iter().map(|seed_option| {
                                html!(
                                <option value={seed_option.label.clone()}>{seed_option.label.clone()}</option>
                            )}).collect::<Html>()}
                        </select>

                        <button class="reset-button" onclick=self.link.callback(|_| Msg::Reset)>{"Reset"}</button>
                    </div>

                    <div class="spacer"></div>

                    <div class="metrics">
                        <div class="metric">
                            <div class="metric-label">{"Edits"}</div>
                            <div class="metric-value">{self.props.modification_count}</div>
                        </div>
                        <div class="metric">
                            <div class="metric-label">{"Steps"}</div>
                            <div class="metric-value">{self.props.step_count}</div>
                        </div>
                        <div class="metric">
                            <div class="metric-label">{"Active"}</div>
                            <div class="metric-value">{self.props.active_count}</div>
                        </div>
                    </div>
                </header>
            </>
        }
    }
}
