use correct_bar::config::{Config, WindowConfig};

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
    ..Default::default()
  };
  correct_bar::run(config)
}
