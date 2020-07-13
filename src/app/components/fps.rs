#[allow(dead_code)]
use log::*;

use wasm_bindgen::{closure::Closure, convert::IntoWasmAbi, prelude::wasm_bindgen, JsValue};
use yew::prelude::*;
use yew::services::{RenderService, Task};


#[derive(Clone, PartialEq)]
pub enum Msg {
    RenderFrame,
}

#[derive(PartialEq, Clone, Properties)]
pub struct Props {
  #[prop_or_default]
  pub oncomplete: Callback<i64>,
}

pub struct FpsDetector {
  props: Props,
  link: ComponentLink<Self>,
  recent_timestamp: f64,
  timings: Vec<f64>,
  render_loop: Option<Box<dyn Task>>,
  finished: bool
}

impl Component for FpsDetector {

  type Message = Msg;
  type Properties = Props;

  fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
      Self {
          props,
          link,
          timings: vec![],
          render_loop: None,
          recent_timestamp: js_sys::Date::now(),
          finished: false
      }
  }

  #[allow(dead_code)]
  fn update(&mut self, message: Self::Message) -> ShouldRender {
      match message {
          Msg::RenderFrame => {
              info!("render happening");

              let now = js_sys::Date::now();
              let current_timing = now - self.recent_timestamp;
              self.timings.push(current_timing);
              self.recent_timestamp = now;

              // 67 is 60 frames plus padding for removing wonky values
              let max_frame_count = 120;
              if self.timings.len() > max_frame_count {

                let skip_frame_count = max_frame_count / 10;
                for i in 0..skip_frame_count {
                  self.timings.remove(0);
                }
                self.timings.shrink_to_fit();

                let sum = self.timings.iter().fold(0.0, |acc, x| acc + x);
                let avg = sum / (self.timings.len() as f64);
                info!("frame rate: {:?} ... {:?}", avg, self.timings);
                info!("timings: {:?} ... {:?}", self.timings[0], self.timings[1]);
                info!("avg: {:?}", (1000.0 / avg));

                info!("timings: {:?} ... {}", self.timings[0], self.timings[0]);

                let margin_of_error: f64 = avg * 0.2;
                let filtered_timings: Vec<f64> = self.timings.clone()
                  .iter().filter(|timing| {
                    if timing > &&(avg + margin_of_error) {
                      false
                    } else if timing < &&(avg - margin_of_error) {
                      false
                    } else {
                      true
                    }
                  })
                  .collect::<Vec<&f64>>()
                  .iter()
                  .map(|timing| timing.to_owned().to_owned())
                  .collect::<Vec<f64>>();

                let filtered_sum = filtered_timings.iter().fold(0.0, |acc, x| acc + x);
                let filtered_avg = filtered_sum / (filtered_timings.len() as f64);

                info!("filtered_avg: {:?}", (1000.0 / filtered_avg));

                self.props.oncomplete.emit(math::round::half_to_even(1000.0 / filtered_avg, 0) as i64);
                self.finished = true;
              }

          }
      }

      true
  }

  fn change(&mut self, props: Self::Properties) -> ShouldRender {
      self.props = props.clone();
      true
  }

  fn rendered(&mut self, first_render: bool) {
    if self.finished == false {
      self.render_next_frame();
    }
  }

  fn view(&self) -> Html {
      html! {
          <>
          </>
      }
  }
}

impl FpsDetector {
  fn render_next_frame(&mut self) {
    let render_frame = self.link.callback(|_| Msg::RenderFrame);
    let handle = RenderService::new().request_animation_frame(render_frame);
    self.render_loop = Some(Box::new(handle));
  }
}
