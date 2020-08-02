mod components;

use anyhow::Error;
use log::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::{closure::Closure, convert::IntoWasmAbi, prelude::wasm_bindgen, JsValue};
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::storage::{Area, StorageService};
use yew::services::{RenderService, Task};
use yew::virtual_dom::vlist::VList;
use yew::virtual_dom::vnode::VNode;

use crate::app::components::fps::FpsDetector;
use crate::app::components::grid::GameGrid;
use crate::app::components::header::AppHeader;

// use crate::app::core::game::{Cellule, GameState, LifeState};
// use crate::app::core::seeds::{seed_middle_line_starter, seed_pentadecathlon};

use game_of_life_core::core::game::{Cellule, GameState, LifeState};
use game_of_life_core::core::history::History;
#[allow(dead_code)]
use game_of_life_core::core::seeds::seeds::{get_seeds, seed_middle_line_starter, Seed};

const KEY: &str = "yew.gameofdeath.self";

struct EnvVars {
    API_URL_SUBMIT_RESULT: String,
    API_URL_GET_HIGH_SCORES: String,
}

pub struct App {
    link: ComponentLink<Self>,
    storage: StorageService,
    state: State,
    #[allow(unused)]
    render_loop: Option<Box<dyn Task>>,
    send_result_fetch_task: Option<FetchTask>,
    fetch_scores_fetch_task: Option<FetchTask>,
    history: History,
    last_render_timestamp: f64,
    seed_options: Vec<Seed>,
    env_vars: EnvVars,
    max_fps: i64,
    previous_scores: Vec<GetScoresResponseDataItem>,
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
    step_count: i32,
    active_count: i32,
    modifications: Vec<GridModification>,
    current_seed: Seed,
    has_life_high_score: bool,
    has_death_high_score: bool,
    user_name: String,
    has_no_network: bool,
    user_name_is_valid: bool,
    rate: f64,
}

#[derive(Serialize, Deserialize)]
struct Pixel {
    on: bool,
}

#[allow(dead_code)]
pub enum Msg {
    GridClicked((i32, i32)),
    HandleSendResultResponse(Result<ResultResponseData, Error>),
    HandleGetScoresResponse(Result<GetScoresResponseData, Error>),
    HandleGetScoresError,
    SendResult,
    Start,
    StepGame,
    Stop,
    Render,
    HandleSeedChange(Seed),
    DismissScoreModal,
    DismissScoreModalClick(MouseEvent),
    SubmitScore(Event),
    ChangeUserName(String),
    HandleRender,
    HandleRateChange(f64),
    HandleFpsDetection(i64),
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
        .to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendResultPayload {
    game_state: SerializedGameState,
    step_count: i32,
    active_count: i32,
    modifications: Vec<GridModification>,
    seed_label: String,
    user_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetScoresResponseDataItem {
    game_state: SerializedGameState,
    step_count: i32,
    active_count: i32,
    modifications: Vec<GridModification>,
    seed_label: String,
    user_name: String,
    _id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetScoresResponseData {
    scores: Vec<GetScoresResponseDataItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            cellule_neighbors: HashMap::new(),
        };

        let state = State {
            grid,
            game_state,
            is_playing: false,
            is_started: false,
            step_count: 0,
            active_count: 0,
            modifications: vec![],
            current_seed,
            has_life_high_score: false,
            has_death_high_score: false,
            user_name: "".to_string(),
            has_no_network: false,
            user_name_is_valid: false,
            rate: 60.0,
        };

        App {
            link,
            storage,
            state,
            render_loop: None,
            send_result_fetch_task: None,
            fetch_scores_fetch_task: None,
            history,
            last_render_timestamp: js_sys::Date::now(), //Instant::now(),
            seed_options,
            env_vars: App::get_env_vars(),
            max_fps: 60,
            previous_scores: vec![],
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.fetch_scores_fetch_task = Some(self.fetch_scores());
            self.update(Msg::Render);
        }

        self.render_next_frame();
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

                    match self
                        .state
                        .modifications
                        .iter()
                        .find(|&modification| modification.grid_index == index)
                    {
                        Some(_) => {
                            self.state.modifications = self
                                .state
                                .modifications
                                .iter()
                                .cloned()
                                .filter(|modification| modification.grid_index != index)
                                .collect::<Vec<GridModification>>()
                        }
                        _ => self.state.modifications.push(GridModification {
                            grid_index: index,
                            step_index: self.state.step_count,
                        }),
                    }

                    self.state.game_state.toggle_cellule(index as usize);
                }
            }
            Msg::HandleGetScoresResponse(get_scores_response) => {
                self.previous_scores = get_scores_response.unwrap().scores;

                self.state.has_no_network = false;
            }
            Msg::HandleGetScoresError => {
                self.state.has_no_network = true;
            }
            Msg::HandleSendResultResponse(send_result_response) => {
                // info!("send result response {:?}", send_result_response)
            }
            Msg::HandleSeedChange(seed) => {
                self.state.is_started = false;
                self.state.is_playing = false;
                self.state.step_count = 0;

                self.state.current_seed = seed.clone();
                self.state.game_state.set_cellules(seed.cellules);

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
                if self.state.is_playing {
                    #[allow(unused_assignments)]
                    let mut in_endless_loop = false;

                    self.state.step_count += 1;
                    self.state.game_state.step();

                    in_endless_loop = self
                        .history
                        .is_in_endless_loop(self.state.game_state.cellules.clone());

                    if in_endless_loop == true || self.state.step_count > 4000 {
                        // temp
                        self.state.has_life_high_score = self.check_life_score();
                        self.state.has_death_high_score = self.check_death_score();

                        self.state.is_playing = false;
                        self.history.clear_previous_steps();
                        warn!("found endless loop or too many steps");
                        warn!("step: {:?}", self.state.step_count);
                        warn!("modifications count: {:?}", self.state.modifications.len());
                        warn!("modifications: {:?}", self.state.modifications);

                        return true;
                    }
                }
                self.set_active_count();
            }
            Msg::Stop => {
                self.state.is_playing = false;
                self.history.clear_previous_steps();
            }
            Msg::DismissScoreModal => {
                self.state.has_life_high_score = false;
                self.state.has_death_high_score = false;
                self.update(Msg::HandleSeedChange(self.state.current_seed.clone()));
            }
            Msg::DismissScoreModalClick(event) => {
                event.prevent_default();
                self.update(Msg::DismissScoreModal);
            }
            Msg::SubmitScore(event) => {
                event.prevent_default();
                self.state.has_life_high_score = false;
                self.state.has_death_high_score = false;
                // make fetch request
                self.send_result_fetch_task = Some(self.post_results());

                self.update(Msg::HandleSeedChange(self.state.current_seed.clone()));
            }
            Msg::ChangeUserName(user_name) => {
                self.state.user_name = user_name;
                self.state.user_name_is_valid = !containsProfanity(&self.state.user_name);
            }
            Msg::HandleRender => {
                let now = js_sys::Date::now();
                let frame_time = 1000.0 / self.state.rate;
                if self.last_render_timestamp + frame_time < now {
                    self.last_render_timestamp = now;
                    self.update(Msg::StepGame);
                } else {
                    self.update(Msg::Nope);
                }
            }
            Msg::HandleFpsDetection(fps) => {
                self.max_fps = fps;
            }
            Msg::HandleRateChange(rate) => {
                self.state.rate = rate;
            }
            Msg::Nope => {}
        }
        self.storage.store(KEY, Json(&self.state.grid));
        true
    }

    fn view(&self) -> Html {
        let error_styles = if self.state.has_no_network == true {
            ""
        } else {
            "display: none;"
        };

        let game_styles = if self.state.has_no_network == true {
            "display: none;"
        } else {
            ""
        };

        let modification_count = self.state.modifications.len() as i32;

        let has_high_score = self.state.has_life_high_score || self.state.has_death_high_score;

        let has_only_life_high_score =
            self.state.has_life_high_score && !self.state.has_death_high_score;

        let has_only_death_high_score =
            !self.state.has_life_high_score && self.state.has_death_high_score;

        let has_both_high_scores =
            self.state.has_life_high_score && self.state.has_death_high_score;

        html! {
            <>
                <div class="game-of-death-wrapper server-down-wrapper" style={error_styles}>
                    <img src="./clean_up.svg" />
                    <h1>{"Server is down"}</h1>
                    <p>{"Give us some time to dive in and clean up the code."}</p>
                </div>
                <div class="game-of-death-wrapper"  style={game_styles}>
                    <AppHeader
                        step_count={self.state.step_count}
                        active_count={self.state.active_count}
                        modification_count={modification_count}
                        seed_options={self.seed_options.clone()}
                        on_seed_change=self.link.callback(|seed| Msg::HandleSeedChange(seed))
                        on_rate_change=self.link.callback(|rate| Msg::HandleRateChange(rate))
                        max_fps={self.max_fps}
                    ></AppHeader>
                    <GameGrid
                        cellules={self.state.game_state.cellules.clone()}
                        cellules_width={self.state.game_state.cellules_width}
                        cellules_height={self.state.game_state.cellules_height}
                        onclick=self.link.callback(Msg::GridClicked)
                        cellule_neighbors={self.state.game_state.cellule_neighbors.clone()}
                    ></GameGrid>

                    <div class="hacky-spacer"></div>

                    <div class="start-wrapper" hidden={self.state.has_no_network || self.state.is_started } style={game_styles}>
                        <button class="start-button" onclick=self.link.callback(|_|  Msg::Start)>{"Start"}</button>
                    </div>

                    <div class=self.modal_classes() onclick=self.link.callback(|event: MouseEvent|  Msg::DismissScoreModalClick(event))>
                        <div
                            class="new-score-modal"
                            onclick=self.link.callback(|event: MouseEvent|  {
                                event.stop_propagation();
                                Msg::Nope
                            })
                        >

                            {self.if_then_render(
                                has_high_score,
                                html!{
                                    <form onsubmit=self.link.callback(|event: Event|  Msg::SubmitScore(event))>

                                        {self.if_then_render(
                                            has_only_life_high_score,
                                            html!{
                                                <>
                                                    <h2>{"New Life Score"}</h2>
                                                    <p>{"Your cellules lived a long time. Enough to be considered for the high score list after we verify it."}</p>
                                                </>
                                            }
                                        )}
                                        {self.if_then_render(
                                            has_only_death_high_score,
                                            html!{
                                                <>
                                                    <h2>{"New Death Score"}</h2>
                                                    <p>{"Not many cellules were left alive. For a Death score that is good. We will verify it soon and maybe put you on the high score list."}</p>
                                                </>
                                            }
                                        )}
                                        {self.if_then_render(
                                            has_both_high_scores,
                                            html!{
                                                <>
                                                    <h2>{"New Life and Death Score"}</h2>
                                                    <p>{"That is amazing!!! Well, maybe. We need to verify all of the recently submitted results to know for certain."}</p>
                                                </>
                                            }
                                        )}
                                        <div class="metrics">
                                            <div class="metric">
                                                <div class="metric-label">{"Edits"}</div>
                                                <div class="metric-value">{self.state.modifications.len()}</div>
                                            </div>
                                            <div class="metric">
                                                <div class="metric-label">{"Steps"}</div>
                                                <div class="metric-value">{self.state.step_count}</div>
                                            </div>
                                            <div class="metric">
                                                <div class="metric-label">{"Active"}</div>
                                                <div class="metric-value">{self.state.active_count}</div>
                                            </div>
                                        </div>
                                        <div class="name-input-wrapper">
                                            <label>{"Enter Name (4 chars max.)"}
                                                <input
                                                    placeholder="ABCD"
                                                    class="name-input" oninput=self.link.callback(|event: InputData| Msg::ChangeUserName(event.value))
                                                    maxlength="4"
                                                />
                                            </label>
                                        </div>
                                        <div class="modal-buttons">
                                            <input type={"button" }
                                                value={"Ignore"}
                                                class="button ignore-button" onclick=self.link.callback(|event: MouseEvent|  Msg::DismissScoreModalClick(event))
                                            />
                                            <input
                                                type="submit"
                                                class="button submit-button"
                                                disabled={!self.state.user_name_is_valid}
                                                value={"Submit"}
                                            />
                                        </div>
                                    </form>
                                }
                            )}


                            {self.if_then_render(
                                !has_high_score,
                                html!{
                                    <div>
                                        <h2>{"Bummer! Try Again?"}</h2>
                                        <p>{"You didnt get a Life or a Death score. No worries, it didn't cost you anything."}</p>
                                        <input
                                            type="button"
                                            class="button try-again-button"
                                            value={"Try Again"}
                                            onclick=self.link.callback(|event: MouseEvent|  Msg::DismissScoreModalClick(event))
                                        />
                                    </div>
                                }
                            )}
                        </div>
                    </div>

                </div>
                // WTF: why is this not working
                <FpsDetector oncomplete=self.link.callback(|fps| Msg::HandleFpsDetection(fps))></FpsDetector>
            </>
        }
    }
}

impl App {
    pub fn modal_classes(&self) -> String {
        if self.state.step_count > 0 && !self.state.is_playing {
            "overlay".to_string()
        } else {
            "overlay hidden".to_string()
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

    fn fetch_scores(&mut self) -> yew::services::fetch::FetchTask {
        let callback = self.link.callback(
            move |response: Response<Json<Result<GetScoresResponseData, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    Msg::HandleGetScoresResponse(data)
                } else {
                    warn!("HTTP request failure");
                    Msg::HandleGetScoresError
                    // Msg::Nope
                }
            },
        );

        let submit_result_url = self.env_vars.API_URL_GET_HIGH_SCORES.clone();
        let request = Request::get(submit_result_url)
            .header("Content-Type", "application/json")
            .body(Nothing)
            .unwrap();
        FetchService::new().fetch(request, callback).unwrap()
    }

    fn post_results(&mut self) -> yew::services::fetch::FetchTask {
        let callback = self.link.callback(
            move |response: Response<Json<Result<ResultResponseData, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
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
                    "0".to_string()
                } else {
                    "1".to_string()
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
            user_name: self.state.user_name.clone(),
        };

        let payload = Json(&raw_payload);

        let submit_result_url = self.env_vars.API_URL_SUBMIT_RESULT.clone();
        let request = Request::post(submit_result_url)
            .header("Content-Type", "application/json")
            .body(payload)
            .unwrap();
        FetchService::new().fetch(request, callback).unwrap()
    }

    fn get_env_vars() -> EnvVars {
        let mut env_vars = EnvVars {
            API_URL_GET_HIGH_SCORES: "".to_string(),
            API_URL_SUBMIT_RESULT: "".to_string(),
        };

        let entries = js_sys::Object::entries(&js_sys::global())
            .iter()
            .filter(|entry| {
                let entry_array = js_sys::Array::from(entry);

                let key = entry_array.get(0).as_string().unwrap();

                if key == "API_URL_SUBMIT_RESULT" {
                    let value = entry_array.get(1).as_string().unwrap();
                    env_vars.API_URL_SUBMIT_RESULT = value;
                    true
                } else if key == "API_URL_GET_HIGH_SCORES" {
                    let value = entry_array.get(1).as_string().unwrap();
                    env_vars.API_URL_GET_HIGH_SCORES = value;
                    true
                } else {
                    false
                }
            })
            .collect::<js_sys::Array>();

        env_vars
    }

    fn render_next_frame(&mut self) {
        let render_frame = self.link.callback(|_| Msg::HandleRender);
        let handle = RenderService::new().request_animation_frame(render_frame);
        self.render_loop = Some(Box::new(handle));
    }

    fn check_life_score(&mut self) -> bool {
        let mut sorted_seed_scores = self
            .previous_scores
            .iter()
            .cloned()
            .filter(|score| {
                score.seed_label == self.state.current_seed.label
                    && score.modifications.len() == self.state.modifications.len()
            })
            .collect::<Vec<GetScoresResponseDataItem>>();

        sorted_seed_scores.sort_by(|score_a, score_b| {
            score_a.step_count.partial_cmp(&score_b.step_count).unwrap()
        });

        if sorted_seed_scores.len() > 0 {
            let length = std::cmp::min(sorted_seed_scores.len() - 1, 20);

            let (top_scores, _) = sorted_seed_scores.split_at(length);

            let lowest_score = top_scores.last();

            if lowest_score.is_some() {
                // need to check active count for tie breakers
                lowest_score.unwrap().step_count < self.state.step_count
            } else {
                true
            }
        } else {
            true
        }
    }

    fn check_death_score(&mut self) -> bool {
        let mut sorted_seed_scores = self
            .previous_scores
            .iter()
            .cloned()
            .filter(|score| {
                score.seed_label == self.state.current_seed.label
                    && score.modifications.len() == self.state.modifications.len()
            })
            .collect::<Vec<GetScoresResponseDataItem>>();

        sorted_seed_scores.sort_by(|score_a, score_b| {
            score_b
                .active_count
                .partial_cmp(&score_a.active_count)
                .unwrap()
        });

        if sorted_seed_scores.len() > 0 {
            let length = std::cmp::min(sorted_seed_scores.len() - 1, 20);

            let (top_scores, _) = sorted_seed_scores.split_at(length);

            // need to check active count for tie breakers
            let is_death_score = top_scores.last().unwrap().active_count > self.state.active_count;

            is_death_score
        } else {
            true
        }
    }

    fn if_then_render(&self, condition: bool, snippet: VNode) -> VNode {
        if condition {
            snippet
        } else {
            html! {}
        }
    }
}

impl State {
    #[allow(dead_code)]
    fn total(&self) -> usize {
        self.grid.len()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = filter)]
    fn containsProfanity(s: &str) -> bool;
}
