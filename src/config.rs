use crate::bar::{Color, Module, Padding};

#[derive(Clone, Debug, Default)]
pub struct Config {
  pub window:     WindowConfig,
  pub background: Color,
  /// Default padding on every module.
  pub padding:    Padding,
  /// Default font size.
  pub font_size:  f32,

  /// UI scale. All fields in `Config` already have this applied. This should
  /// only be used if drawing rectangles by hand in a module.
  pub scale: f64,

  pub modules_left:   Vec<Module>,
  pub modules_middle: Vec<Module>,
  pub modules_right:  Vec<Module>,
}

#[derive(Clone, Debug, Default)]
pub struct WindowConfig {
  pub height:         u32,
  /// All settings should be set assuming the window monitor will be this wide.
  /// Then, the bar will scale all settings relative to how large the monitor
  /// actually is. So, setting this to `1920` would make all scaling settings
  /// doubled on a `3840` pixel wide display.
  pub expected_width: u32,

  pub margin_top:    u32,
  pub margin_left:   u32,
  pub margin_right:  u32,
  pub margin_bottom: u32,
}

impl Config {
  pub fn apply_scaling_for_width(&mut self, width: u32) {
    let scale = width as f64 / self.window.expected_width as f64;
    self.apply_scale(scale);
  }

  pub fn apply_scale(&mut self, scale: f64) {
    if self.scale != 0.0 {
      panic!("UI scaling of {} already applied", self.scale);
    }
    macro_rules! scale {
      ( $expr:expr, $ty:ty ) => {
        $expr = (($expr as f64) * scale) as $ty;
      };
    }
    scale!(self.window.height, u32);
    scale!(self.window.margin_top, u32);
    scale!(self.window.margin_left, u32);
    scale!(self.window.margin_right, u32);
    scale!(self.window.margin_bottom, u32);
    scale!(self.padding.top, u32);
    scale!(self.padding.left, u32);
    scale!(self.padding.right, u32);
    scale!(self.padding.bottom, u32);
    scale!(self.font_size, f32);
  }
}
