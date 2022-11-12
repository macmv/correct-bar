use correct_bar::{
  bar::{Color, Padding},
  config::{Config, WindowConfig},
};

pub fn run() {
  let (modules_left, modules_middle, modules_right) = super::modules();
  let config = Config {
    window: WindowConfig {
      margin_top: 0,
      margin_left: 0,
      margin_right: 0,
      margin_bottom: 0,
      width: 3840,
      height: 50,
      ..Default::default()
    },
    modules_left,
    modules_middle,
    modules_right,
    padding: Padding { left: 20, right: 20, top: 0, bottom: 0 },
    background: Color::from_hex(0x464646),
    font_size: 48.0,
    ..Default::default()
  };
  correct_bar::run(config)
}
