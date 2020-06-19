mod components;

use anyhow::Error;
use log::*;
use serde_derive::{Deserialize, Serialize};
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::storage::{Area, StorageService};
use yew::services::{RenderService, Task};

use crate::app::components::grid::GameGrid;
use crate::app::components::header::AppHeader;

// use crate::app::core::game::{Cellule, GameState, LifeState};
// use crate::app::core::seeds::{seed_middle_line_starter, seed_pentadecathlon};

use game_of_life_core::core::game::{Cellule, GameState, LifeState};
use game_of_life_core::core::history::History;
#[allow(dead_code)]
use game_of_life_core::core::seeds::seeds::{get_seeds, seed_middle_line_starter, Seed};

const KEY: &str = "yew.gameofdeath.self";

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
    #[allow(unused)]
    render_loop: Option<Box<dyn Task>>,
    send_result_fetch_task: Option<FetchTask>,
    history: History,
    last_render_timestamp: f64,
    seed_options: Vec<Seed>,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct GridModification {
    step_index: i32,
    grid_index: i32,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    grid: Vec<Vec<Pixel>>,
    game_state: GameState,
    is_playing: bool,
    is_started: bool,
    modification_count: i32,
    step_count: i32,
    active_count: i32,
    modifications: Vec<GridModification>,
    current_seed: Seed,
}

#[derive(Serialize, Deserialize)]
struct Pixel {
    on: bool,
}

#[allow(dead_code)]
pub enum Msg {
    GridClicked((i32, i32)),
    HandleSendResultResponse(Result<ResultResponseData, Error>),
    SendResult,
    Start,
    StepGame,
    Stop,
    Render,
    HandleSeedChange(Seed),
    Nope,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultResponseData {
    message: String,
}

impl From<SendResultPayload> for std::result::Result<std::string::String, anyhow::Error> {
    fn from(payload: SendResultPayload) -> Self {
        Ok(format!(
            "{{
                \"game_state\": \"{:?}\",
                \"step_count\": \"{:?}\",
                \"active_count\": \"{:?}\",
                \"modifications\": \"{:?}\",
                \"seed_label\": \"{:?}\"
            }}",
            payload.game_state,
            payload.step_count,
            payload.active_count,
            payload.modifications,
            payload.seed_label
        )
        .to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendResultPayload {
    game_state: SerializedGameState,
    step_count: i32,
    active_count: i32,
    modifications: Vec<GridModification>,
    seed_label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedGameState {
    cellules: String,
    pub active: bool,
    pub cellules_width: usize,
    pub cellules_height: usize,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let storage = StorageService::new(Area::Local).unwrap();
        let grid = {
            if let Json(Ok(restored_grid)) = storage.restore(KEY) {
                restored_grid
            } else {
                Vec::new()
            }
        };

        let history = History {
            previous_steps: vec![],
        };

        let seed_options = get_seeds();

        let current_seed = seed_options[0].clone();

        let game_state = GameState {
            active: false,
            cellules: current_seed.cellules.clone(),
            cellules_width: 50,
            cellules_height: 40,
        };

        let state = State {
            grid,
            game_state,
            is_playing: false,
            is_started: false,
            modification_count: 0,
            step_count: 0,
            active_count: 0,
            modifications: vec![],
            current_seed,
        };

        App {
            link,
            storage,
            state,
            render_loop: None,
            send_result_fetch_task: None,
            history,
            last_render_timestamp: js_sys::Date::now(), //Instant::now(),
            seed_options,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.update(Msg::Render);
        }

        if self.last_render_timestamp + 15.0 < js_sys::Date::now() {
            self.render_next_frame();
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GridClicked((column_number, row_number)) => {
                if self.state.is_started == false {
                    let index =
                        row_number * (self.state.game_state.cellules_width as i32) + column_number;

                    self.state.modifications.push(GridModification {
                        grid_index: index,
                        step_index: self.state.step_count,
                    });

                    self.state.modification_count += 1;

                    // info!("{:?} {:?} {:?}", column_number, row_number, index);

                    self.state.game_state.toggle_cellule(index as usize);
                }
            }
            Msg::HandleSendResultResponse(send_result_response) => {
                info!("send result response {:?}", send_result_response)
            }
            Msg::HandleSeedChange(seed) => {
                self.state.modification_count = 0;
                self.state.step_count = 0;

                self.state.game_state.set_cellules(seed.cellules);

                self.state.is_started = false;
                self.set_active_count();
                self.history.clear_previous_steps();
                self.state.modifications = vec![];
            }
            Msg::Render => info!("send result response"),
            Msg::SendResult => {}
            Msg::Start => {
                self.state.is_playing = true;
                self.state.is_started = true;
                self.history.clear_previous_steps();
            }
            Msg::StepGame => {
                // info!("step clicked");
                if self.state.is_playing {
                    #[allow(unused_assignments)]
                    let mut in_endless_loop = false;

                    self.state.step_count += 1;
                    self.state.game_state.step();

                    in_endless_loop = self
                        .history
                        .is_in_endless_loop(self.state.game_state.cellules.clone());

                    if in_endless_loop == true || self.state.step_count > 4000 {
                        self.state.is_playing = false;
                        self.history.clear_previous_steps();
                        warn!("found endless loop or too many steps");
                        warn!("step: {:?}", self.state.step_count);
                        warn!("modifications count: {:?}", self.state.modifications.len());
                        warn!("modifications: {:?}", self.state.modifications);

                        // make fetch request
                        self.send_result_fetch_task = Some(self.fetch_json());

                        return true;
                    }
                }
                self.set_active_count();
            }
            Msg::Stop => {
                self.state.is_playing = false;
                self.history.clear_previous_steps();
            }
            Msg::Nope => {}
        }
        self.storage.store(KEY, Json(&self.state.grid));
        true
    }

    fn view(&self) -> Html {
        // info!("rendered!");
        html! {
            <div class="game-of-death-wrapper">
                <AppHeader
                    step_count={self.state.step_count}
                    active_count={self.state.active_count}
                    modification_count={self.state.modification_count}
                    seed_options={self.seed_options.clone()}
                    on_seed_change=self.link.callback(|seed| Msg::HandleSeedChange(seed))
                ></AppHeader>
                <GameGrid
                    cellules={self.state.game_state.cellules.clone()}
                    cellules_width={self.state.game_state.cellules_width}
                    cellules_height={self.state.game_state.cellules_height}
                    onclick=self.link.callback(Msg::GridClicked)
                ></GameGrid>
                <div class="hacky-spacer"></div>
                <div class=self.start_wrapper_classes()>
                    <button class="start-button" onclick=self.link.callback(|_|  Msg::Start)>{"Start"}</button>
                </div>

            </div>
        }
    }
}

impl App {
    pub fn start_wrapper_classes(&self) -> String {
        if self.state.is_started {
            "start-wrapper started".to_owned()
        } else {
            "start-wrapper".to_owned()
        }
    }

    fn set_active_count(&mut self) -> () {
        let mut active_count = 0;
        for cellule in self.state.game_state.cellules.iter() {
            if cellule.life_state == LifeState::Alive {
                active_count += 1;
            }
        }
        self.state.active_count = active_count;
    }

    fn fetch_json(&mut self) -> yew::services::fetch::FetchTask {
        let callback = self.link.callback(
            move |response: Response<Json<Result<ResultResponseData, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                info!("META: {:?}, {:?}", meta, data);
                if meta.status.is_success() {
                    Msg::HandleSendResultResponse(data)
                } else {
                    warn!("HTTP request failure");
                    Msg::Nope
                }
            },
        );

        let serialized_cellules = self
            .state
            .game_state
            .cellules
            .iter()
            .map(|cellule| {
                if cellule.life_state == LifeState::Dead {
                    "0".to_owned()
                } else {
                    "1".to_owned()
                }
            })
            .collect::<Vec<String>>()
            .join("");

        let raw_payload = SendResultPayload {
            modifications: self.state.modifications.clone(),
            step_count: self.state.step_count,
            active_count: self.state.active_count,
            seed_label: self.state.current_seed.label.clone(),
            game_state: SerializedGameState {
                cellules: serialized_cellules,
                cellules_width: self.state.game_state.cellules_width,
                cellules_height: self.state.game_state.cellules_height,
                active: false,
            },
        };
        let payload = Json(&raw_payload);
        let request = Request::post("http://localhost:3000/prod/submit-result")
            .header("Content-Type", "application/json")
            .body(payload)
            .unwrap();
        FetchService::new().fetch(request, callback).unwrap()
    }

    fn render_next_frame(&mut self) {
        let render_frame = self.link.callback(|_| Msg::StepGame);
        let handle = RenderService::new().request_animation_frame(render_frame);
        self.render_loop = Some(Box::new(handle));
    }
}

impl State {
    #[allow(dead_code)]
    fn total(&self) -> usize {
        self.grid.len()
    }
}
