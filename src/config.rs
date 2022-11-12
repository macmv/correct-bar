use crate::bar::{Color, Module};

#[derive(Default)]
pub struct Config {
  pub window:     WindowConfig,
  pub background: Color,

  pub modules_left:   Vec<Module>,
  pub modules_middle: Vec<Module>,
  pub modules_right:  Vec<Module>,
}

#[derive(Default)]
pub struct WindowConfig {
  pub width:  u32,
  pub height: u32,

  pub margin_top:    u32,
  pub margin_left:   u32,
  pub margin_right:  u32,
  pub margin_bottom: u32,
}
