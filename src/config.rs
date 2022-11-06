#[derive(Default)]
pub struct Config {
  pub window: WindowConfig,
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
