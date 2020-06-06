use crate::app::core::game::{Cellule, LifeState};
#[allow(dead_code)]
use log::*;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryInto;
use std::iter::Iterator;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, ToString};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, EventTarget, HtmlCanvasElement, HtmlElement};
use yew::format::Json;
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

        let translation_ratio = ((BASE_CELLULE_SIZE as f32) * self.props.cellules_width as f32)
          / canvas_element.client_width() as f32;

        self.render_canvas(canvas_element);

        debug!(
          "offset: x{:?} y{:?}",
          mouse_event.offset_x(),
          mouse_event.offset_y()
        );

        let column_number =
          ((mouse_event.offset_x() as f32) * translation_ratio / BASE_CELLULE_SIZE as f32) as i32;
        let row_number =
          ((mouse_event.offset_y() as f32) * translation_ratio / BASE_CELLULE_SIZE as f32) as i32;

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
    ctx.set_fill_style(&JsValue::from_str("gray"));

    for cellule_index in 0..2000 {
      let row_number = cellule_index / self.props.cellules_width;
      let column_number = cellule_index % self.props.cellules_width;

      let x = BASE_CELLULE_SIZE * (column_number as i32);
      let y = BASE_CELLULE_SIZE * (row_number as i32);

      if self.props.cellules[cellule_index].life_state == LifeState::Alive {
        ctx.set_fill_style(&JsValue::from_str("red"));
        ctx.fill_rect(
          x as f64,
          y as f64,
          BASE_CELLULE_SIZE as f64,
          BASE_CELLULE_SIZE as f64,
        );
        ctx.set_fill_style(&JsValue::from_str("gray"));
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

fn get_payload_later(payload_callback: Callback<String>) {
  let callback =
    Closure::once_into_js(move |payload: String| payload_callback.emit("Math.random()".to_owned()));
  get_payload_later_js(callback);
}
