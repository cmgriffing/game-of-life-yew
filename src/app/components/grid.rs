// use crate::app::core::game::{Cellule, LifeState};
use game_of_life_core::core::game::{Cellule, LifeState};

use crate::utils::colors::*;

#[allow(dead_code)]
use log::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

const BASE_CELLULE_SIZE: i32 = 20;

pub enum Msg {
  Click(MouseEvent),
}

#[derive(PartialEq, Clone, Properties)]
pub struct Props {
  #[prop_or_default]
  pub inset: bool,
  #[prop_or_default]
  pub children: Children,
  #[prop_or_default]
  pub cellules: Vec<Cellule>,
  #[prop_or_default]
  pub cellules_width: usize,
  #[prop_or_default]
  pub cellules_height: usize,

  #[prop_or_default]
  pub onclick: Callback<(i32, i32)>,
}

pub struct GameGrid {
  props: Props,
  canvas_ref: NodeRef,
  link: ComponentLink<Self>,
}

impl Component for GameGrid {
  type Message = Msg;
  type Properties = Props;

  fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
    Self {
      props,
      canvas_ref: NodeRef::default(),
      link,
    }
  }

  fn update(&mut self, msg: Self::Message) -> ShouldRender {
    match msg {
      Msg::Click(mouse_event) => {
        let canvas_element = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();

        let translation_ratio_x = ((BASE_CELLULE_SIZE as f32) * self.props.cellules_width as f32)
          / canvas_element.client_width() as f32;

        let translation_ratio_y = ((BASE_CELLULE_SIZE as f32) * self.props.cellules_height as f32)
            / canvas_element.client_height() as f32;

        self.render_canvas(canvas_element);

        debug!(
          "offset: x{:?} y{:?}",
          mouse_event.offset_x(),
          mouse_event.offset_y()
        );

        let column_number =
          ((mouse_event.offset_x() as f32) * translation_ratio_x / BASE_CELLULE_SIZE as f32) as i32;
        let row_number =
          ((mouse_event.offset_y() as f32) * translation_ratio_y / BASE_CELLULE_SIZE as f32) as i32;

        info!(
          " click happened in grid {:?} {:?}",
          column_number, row_number
        );

        self.props.onclick.emit((column_number, row_number));

        true
      }
    }
  }

  fn change(&mut self, props: Self::Properties) -> ShouldRender {
    self.props = props;

    // info!("change event in grid");

    let canvas_element = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();

    self.render_canvas(canvas_element);

    true
  }

  fn view(&self) -> Html {
    let canvas_height = (self.props.cellules_height as i32) * BASE_CELLULE_SIZE;
    let canvas_width = (self.props.cellules_width as i32) * BASE_CELLULE_SIZE;

    html! {
      <>
        <canvas
          onclick=self.link.callback(|event: MouseEvent| Msg::Click(event))
          class="game-board"
          ref=self.canvas_ref.clone()
          height=canvas_height
          width=canvas_width
        ></canvas>
      </>
    }
  }
}

impl GameGrid {
  fn render_canvas(&self, canvas_element: HtmlCanvasElement) -> HtmlCanvasElement {
    let ctx = CanvasRenderingContext2d::from(JsValue::from(
      canvas_element.get_context("2d").unwrap().unwrap(),
    ));
    ctx.set_fill_style(&JsValue::from_str("#aaaadd"));

    let gradient_manager = GradientManager::new(
      Color {
        red: 246.0,
        green: 157.0,
        blue: 60.0,
      },
      Color {
        red: 63.0,
        green: 135.0,
        blue: 166.0,
      },
    );

    for cellule_index in 0..2000 {
      let row_number = cellule_index / self.props.cellules_width;
      let column_number = cellule_index % self.props.cellules_width;

      let x = (BASE_CELLULE_SIZE as f32) * (column_number as f32);
      let y = (BASE_CELLULE_SIZE as f32) * (row_number as f32);

      if self.props.cellules[cellule_index].life_state == LifeState::Alive {
        // "rgb()"
        // let new_color = format!("rgb({}, {}, {})", newRed, newGreen, newBlue);

        let added_indexes = (column_number + row_number) as f32;
        let progress_percentage = added_indexes / 100.0;
        let color = gradient_manager.interpolate_colors(progress_percentage);
        let new_color = format!("rgb({}, {}, {})", color.red, color.green, color.blue);
        ctx.set_fill_style(&JsValue::from_str(new_color.as_str()));
        ctx.fill_rect(
          x as f64,
          y as f64,
          BASE_CELLULE_SIZE as f64,
          BASE_CELLULE_SIZE as f64,
        );
        ctx.set_fill_style(&JsValue::from_str("#aaaadd"));
      } else {
        ctx.fill_rect(
          x as f64,
          y as f64,
          BASE_CELLULE_SIZE as f64,
          BASE_CELLULE_SIZE as f64,
        );
      }
    }

    canvas_element
  }
}

#[wasm_bindgen]
extern "C" {
  fn get_payload() -> String;
}

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_name = "get_payload_later")]
  fn get_payload_later_js(payload_callback: JsValue);
}

#[allow(dead_code)]
fn get_payload_later(payload_callback: Callback<String>) {
  #[allow(unused_variables)]
  let callback =
    Closure::once_into_js(move |payload: String| payload_callback.emit("Math.random()".to_owned()));
  get_payload_later_js(callback);
}
