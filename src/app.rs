mod components;
mod core;

use anyhow::Error;
use log::*;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, ToString};
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::interval::{IntervalService, IntervalTask};
use yew::services::storage::{Area, StorageService};
use yew::services::Task;

use crate::app::components::grid::GameGrid;
use crate::app::components::header::AppHeader;

use crate::app::core::game::{Cellule, GameState, LifeState};
use crate::app::core::seeds::{seed_middle_line_starter, seed_pentadecathlon};

const KEY: &str = "yew.gameofdeath.self";
const MAXIMUM_LOOP_TURN_COUNT: usize = 15;

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
    #[allow(unused)]
    interval_handle: Box<dyn Task>,
    previous_steps: Vec<Vec<Cellule>>,
    send_result_fetch_task: Option<FetchTask>,
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
    modification_count: i32,
    step_count: i32,
    active_count: i32,
    modifications: Vec<GridModification>,
}

#[derive(Serialize, Deserialize)]
struct Pixel {
    on: bool,
}

pub enum Msg {
    GridClicked((i32, i32)),
    HandleSendResultResponse(Result<ResultResponseData, Error>),
    RandomSeed,
    SendResult,
    Start,
    StepClicked,
    Stop,
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
            }}",
            payload.game_state, payload.step_count, payload.active_count, payload.modifications
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

        let game_state = GameState {
            active: false,
            cellules: vec![
                Cellule {
                    life_state: LifeState::Dead
                };
                2000
            ],
            cellules_width: 50,
            cellules_height: 40,
        };

        let interval_callback = link.callback(|_| Msg::StepClicked);

        let state = State {
            grid,
            game_state,
            is_playing: false,
            modification_count: 0,
            step_count: 0,
            active_count: 0,
            modifications: vec![],
        };
        App {
            link,
            storage,
            state,
            interval_handle: Box::new(
                IntervalService::new().spawn(Duration::from_millis(15), interval_callback),
            ),
            previous_steps: vec![],
            send_result_fetch_task: None,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GridClicked((column_number, row_number)) => {
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
            Msg::HandleSendResultResponse(send_result_response) => {
                info!("send result response {:?}", send_result_response)
            }
            Msg::RandomSeed => {
                self.state.modification_count = 0;
                self.state.step_count = 0;
                let cellules = seed_middle_line_starter(
                    self.state.game_state.cellules_width as i32,
                    self.state.game_state.cellules_height as i32,
                );
                self.state.game_state.set_cellules(cellules);

                self.set_active_count();
                self.previous_steps = vec![];
                self.state.modifications = vec![];
            }
            Msg::SendResult => {}
            Msg::Start => {
                self.state.is_playing = true;
                self.previous_steps = vec![];
            }
            Msg::StepClicked => {
                // info!("step clicked");
                if self.state.is_playing {
                    let mut in_endless_loop = false;

                    self.state.step_count += 1;
                    self.state.game_state.step();

                    for previous_step in self.previous_steps.iter() {
                        if self.game_grids_are_identical(
                            previous_step.to_vec(),
                            self.state.game_state.cellules.clone(),
                        ) {
                            in_endless_loop = true;
                        }
                    }

                    if in_endless_loop == true {
                        self.state.is_playing = false;
                        self.previous_steps = vec![];
                        warn!("found endless loop");
                        warn!("step: {:?}", self.state.step_count);
                        warn!("modifications count: {:?}", self.state.modifications.len());
                        warn!("modifications: {:?}", self.state.modifications);

                        // make fetch request
                        self.send_result_fetch_task = Some(self.fetch_json());

                        return true;
                    }

                    let previous_steps_count = self.previous_steps.len();
                    if previous_steps_count >= MAXIMUM_LOOP_TURN_COUNT {
                        let extra_count = previous_steps_count - MAXIMUM_LOOP_TURN_COUNT;
                        self.previous_steps.drain(0..extra_count);
                    }
                    self.previous_steps
                        .push(self.state.game_state.cellules.clone());
                }
                self.set_active_count();
            }
            Msg::Stop => {
                self.state.is_playing = false;
                self.previous_steps = vec![];
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
                <AppHeader></AppHeader>
                <GameGrid
                    cellules={self.state.game_state.cellules.clone()}
                    cellules_width={self.state.game_state.cellules_width}
                    cellules_height={self.state.game_state.cellules_height}
                    onclick=self.link.callback(Msg::GridClicked)
                ></GameGrid>
                <button onclick=self.link.callback(|_|  Msg::StepClicked)>{"Step"}</button>
                <button onclick=self.link.callback(|_|  Msg::RandomSeed)>{"Random Seed"}</button>
                <button onclick=self.link.callback(|_|  Msg::Start)>{"Start"}</button>
                <button onclick=self.link.callback(|_|  Msg::Stop)>{"Stop"}</button>
                <span>{"Modifications: "} {self.state.modification_count}</span>
                <span>{"Steps: "} {self.state.step_count}</span>
                <span>{"Active: "} {self.state.active_count}</span>
                <span>{"previous frame count: "} {self.previous_steps.len()}</span>
            </div>
        }
    }
}

impl App {
    fn set_active_count(&mut self) -> () {
        let mut active_count = 0;
        for cellule in self.state.game_state.cellules.iter() {
            if cellule.life_state == LifeState::Alive {
                active_count += 1;
            }
        }
        self.state.active_count = active_count;
    }

    fn game_grids_are_identical(&self, grid_a: Vec<Cellule>, grid_b: Vec<Cellule>) -> bool {
        let length_a = grid_a.len();
        let length_b = grid_b.len();

        // info!("grid_a {:?} {:?}", grid_a.len(), grid_a[1].alive());
        // info!("grid_b {:?} {:?}", grid_b.len(), grid_b[1].alive());

        if length_a != length_b {
            warn!("lengths not equal");
            return false;
        }

        for index in 0..(length_a - 1) {
            if grid_a[index].life_state != grid_b[index].life_state {
                return false;
            }
        }

        true
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
}

impl State {
    fn total(&self) -> usize {
        self.grid.len()
    }
}
