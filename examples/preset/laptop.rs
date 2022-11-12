use correct_bar::config::{Config, WindowConfig};

pub fn run() {
  let (modules_left, modules_middle, modules_right) = super::modules();
  let config = Config {
    window: WindowConfig {
      margin_top: 40,
      margin_left: 10,
      margin_right: 10,
      margin_bottom: 10,
      width: 1920 - 20,
      height: 50,
      ..Default::default()
    },
    modules_left,
    modules_middle,
    modules_right,
    ..Default::default()
  };
  correct_bar::run(config)
}
