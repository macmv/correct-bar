use correct_bar::{
  bar::{Color, Padding},
  config::{Config, WindowConfig},
};

pub fn run() {
  let (modules_left, modules_middle, modules_right) = super::modules(false);
  let config = Config {
    window: WindowConfig {
      height:         25,
      expected_width: 1920,
      margin_top:     40,
      margin_left:    10,
      margin_right:   10,
      margin_bottom:  10,
    },
    modules_left,
    modules_middle,
    modules_right,
    padding: Padding { left: 5, right: 5, top: 0, bottom: 0 },
    background: Color::from_hex(0x444444),
    font_size: 24.0,
    ..Default::default()
  };
  correct_bar::run(config)
}
