use correct_bar::config::{Config, WindowConfig};

pub fn run() {
  let (modules_left, modules_middle, modules_right) = super::modules();
  let config = Config {
    window: WindowConfig {
      margin_top: 16,
      margin_left: 16,
      margin_right: 16,
      margin_bottom: 16,
      width: 3840 - 32,
      height: 50,
      ..Default::default()
    },
    modules_left,
    ..Default::default()
  };
  correct_bar::run(config)
}
