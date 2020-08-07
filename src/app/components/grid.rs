use std::collections::HashMap;
// use crate::app::core::game::{Cellule, LifeState};
use game_of_life_core::core::game::{Cellule, LifeState};
use yewtil::NeqAssign;

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
  pub cellule_neighbors: HashMap<usize, Vec<Cellule>>,

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

        self.props.onclick.emit((column_number, row_number));

        true
      }
    }
  }

  fn change(&mut self, props: Self::Properties) -> ShouldRender {
    if self.props.neq_assign(props) {
      let canvas_element = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();

      self.render_canvas(canvas_element);
      true
    } else {
      false
    }
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

    ctx.fill_rect(
      0.0,
      0.0,
      BASE_CELLULE_SIZE as f64 * (self.props.cellules_width as f64),
      BASE_CELLULE_SIZE as f64 * (self.props.cellules_height as f64),
    );

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

        ctx.begin_path();

        let radius = (BASE_CELLULE_SIZE as f64) / 2.0;

        ctx
          .ellipse(
            x as f64 + radius,
            y as f64 + radius,
            radius,
            radius,
            0.0,
            0.0,
            6.29,
          )
          .unwrap();
        ctx.fill();
        ctx.close_path();

        self.draw_neighbors(&ctx, cellule_index, x as f64, y as f64);

        ctx.set_fill_style(&JsValue::from_str("#aaaadd"));
      }
    }

    canvas_element
  }

  fn draw_neighbors(
    &self,
    ctx: &CanvasRenderingContext2d,
    cellule_index: usize,
    cellule_x: f64,
    cellule_y: f64,
  ) {
    let neighbors = self.props.cellule_neighbors.get(&cellule_index);

    if neighbors.is_some() {
      neighbors
        .unwrap()
        .iter()
        .enumerate()
        .for_each(|(neighbor_index, cellule)| {
          if cellule.life_state == LifeState::Alive {
            self.draw_neighbor_cell(ctx, cellule, neighbor_index, cellule_x, cellule_y)
          }
        });
    }
  }

  fn draw_neighbor_cell(
    &self,
    ctx: &CanvasRenderingContext2d,
    cellule: &Cellule,
    neighbor_index: usize,
    cellule_x: f64,
    cellule_y: f64,
  ) {
    let radius = (BASE_CELLULE_SIZE as f64) / 2.0;
    let modifiers = match neighbor_index {
      0 => (-1.0, -1.0),
      1 => (0.0, -1.0),
      2 => (1.0, -1.0),
      3 => (1.0, 0.0),
      4 => (1.0, 1.0),
      5 => (0.0, 1.0),
      6 => (-1.0, 1.0),
      7 => (-1.0, 0.0),
      _ => (0.0, 0.0),
    };

    let (modifier_x, modifier_y) = modifiers;

    self.draw_neighbor_ellipse(
      ctx,
      cellule_x + (radius * modifier_x),
      cellule_y + (radius * modifier_y),
    )
  }

  fn draw_neighbor_ellipse(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
    let staggered_step = (((js_sys::Date::now() / 200.0) + x + y).sin() / 5.0) + 0.1;
    let step = staggered_step;

    // let unison_step = ((js_sys::Date::now() / 200.0).sin() / 5.0) + 0.1;
    // let step = unison_step;

    ctx.begin_path();

    let radius = (BASE_CELLULE_SIZE as f64) / 2.0;

    // let radius_modifier = 1.2 + step + ((x as f64 + y as f64).sin() / 5.0);

    let radius_modifier = 1.2 + step;

    ctx
      .ellipse(
        x as f64 + radius,
        y as f64 + radius,
        radius * radius_modifier,
        radius * radius_modifier,
        0.0,
        0.0,
        6.29,
      )
      .unwrap();
    ctx.fill();
    ctx.close_path();
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
