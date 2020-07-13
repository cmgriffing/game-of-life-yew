#[allow(dead_code)]
use log::*;

#[allow(unused_imports)]
use simplelog::*;

#[derive(Copy, Clone)]
pub struct Color {
  pub red: f32,

  pub green: f32,

  pub blue: f32,
}

pub struct GradientManager {
  start_color: Color,
  end_color: Color,
}

impl GradientManager {
  pub fn new(start_color: Color, end_color: Color) -> GradientManager {
    GradientManager {
      start_color,
      end_color,
    }
  }

  pub fn interpolate_colors(&self, progress_percentage: f32) -> Color {
    let red_diff = self.start_color.red - self.end_color.red;
    let green_diff = self.start_color.green - self.end_color.green;
    let blue_diff = self.start_color.blue - self.end_color.blue;

    let red_delta = red_diff * progress_percentage;
    let green_delta = green_diff * progress_percentage;
    let blue_delta = blue_diff * progress_percentage;

    let red = self.start_color.red - red_delta;
    let green = self.start_color.green - green_delta;
    let blue = self.start_color.blue - blue_delta;

    Color { red, green, blue }
  }
}

#[cfg(test)]
mod testing {

  use super::*;

  #[test]
  fn test_color_difference() {
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    let start_color = Color {
      red: 246.0,
      green: 157.0,
      blue: 60.0,
    };
    let end_color = Color {
      red: 63.0,
      green: 135.0,
      blue: 166.0,
    };
    let gradient_manager = GradientManager::new(start_color, end_color);

    let gradient_at_0 = gradient_manager.interpolate_colors(0.0);
    let gradient_at_100 = gradient_manager.interpolate_colors(1.0);
    let gradient_at_39 = gradient_manager.interpolate_colors(0.39);
    let gradient_at_40 = gradient_manager.interpolate_colors(0.40);

    assert_eq!(gradient_at_0.red, start_color.red);
    assert_eq!(gradient_at_100.red, end_color.red);

    assert_ne!(gradient_at_39.red, gradient_at_40.red);
  }
}
