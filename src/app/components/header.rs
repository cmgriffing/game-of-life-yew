#[allow(dead_code)]
use log::*;
use yew::format::Json;
use yew::prelude::*;
use yew::services::storage::{Area, StorageService};

use yewtil::NeqAssign;

use crate::app::components::fps::FpsDetector;
use game_of_life_core::core::seeds::seeds::Seed;

#[derive(Clone, PartialEq)]
pub enum Msg {
    Reset,
    SeedChanged(usize),
    UpdateRate(String),
    ToggleConfig,
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
    pub on_rate_change: Callback<f64>,

    #[prop_or_default]
    pub seed_options: Vec<Seed>,

    #[prop_or_default]
    pub max_fps: i64,
}

pub struct AppHeader {
    props: Props,
    link: ComponentLink<Self>,
    current_seed: Seed,
    rate: f64,
    created_timestamp: f64,
    showing_config: bool,
}

impl Component for AppHeader {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        // info!("CREATED");
        Self {
            props,
            link,
            current_seed: Seed {
                cellules: vec![],
                label: "Loading".to_owned(),
            },
            rate: 60.0,
            created_timestamp: js_sys::Date::now(),
            showing_config: false,
        }
    }

    #[allow(dead_code)]
    fn update(&mut self, message: Self::Message) -> ShouldRender {
        // info!("UPDATE");
        match message {
            Msg::Reset => {
                self.props.on_seed_change.emit(self.current_seed.clone());
            }
            Msg::SeedChanged(seed_option_index) => {
                let seed_option = self.props.seed_options[seed_option_index].clone();
                self.current_seed = seed_option.clone();
                self.props.on_seed_change.emit(seed_option);
            }
            Msg::UpdateRate(rate) => {
                let rate_f64 = rate.parse::<f64>().unwrap();
                self.props.on_rate_change.emit(rate_f64);
                self.rate = rate_f64;
            }
            Msg::ToggleConfig => {
                self.showing_config = !self.showing_config;
            }
        }

        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.neq_assign(props) {
            // info!("CHANGE");
            if self.current_seed.label == "Loading".to_owned() {
                self.current_seed = self.props.seed_options[0].clone();
                self.props
                    .on_seed_change
                    .emit(self.props.seed_options[0].clone());
            }

            if self.rate > self.props.max_fps as f64 {
                self.rate = self.props.max_fps as f64;
                self.props.on_rate_change.emit(self.rate);
            }

            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let showing = if self.showing_config { "showing" } else { "" };
        // info!("VIEW");
        html! {
            <>
                <h1>{ "Cellule Life" }</h1>
                <button class="mobile-menu-button" onclick=self.link.callback(|_| Msg::ToggleConfig)>{"☰"}<span class="sr-only">
                    {"Show or Hide menu"}
                </span></button>
                <header class="main-header">
                    <div class=format!("mobile-config-row controls {}", showing)>
                        <select onchange=self.link.callback(|event: ChangeData| match event {
                            ChangeData::Select(element) => {
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

                    <div class=format!("mobile-config-row slider-section {}", showing)>
                        <label labelFor="rate">{"Rate: "} {self.rate}{"fps"}</label>
                        <div class="slider-component">
                            <div class="slider-label-start">{"10"}</div>
                            <input
                                type="range"
                                name="rate"
                                id="rate"
                                min="10"
                                max={self.props.max_fps}
                                step="10"
                                oninput=self.link.callback(|event: InputData| Msg::UpdateRate(event.value))
                            />
                            <div class="slider-label-end">{self.props.max_fps}</div>
                        </div>
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
