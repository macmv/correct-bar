use crate::bar::{Color, Module, Padding};

#[derive(Clone, Default)]
pub struct Config {
  pub window:     WindowConfig,
  pub background: Color,
  /// Default padding on every module.
  pub padding:    Padding,
  /// Default font size.
  pub font_size:  f32,

  pub modules_left:   Vec<Module>,
  pub modules_middle: Vec<Module>,
  pub modules_right:  Vec<Module>,
}

#[derive(Clone, Default)]
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
