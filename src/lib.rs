mod backend;
mod config;

pub use config::Config;

pub fn run() {
  let config = Config {
    window: config::WindowConfig {
      margin_top: 50,
      margin_left: 50,
      margin_right: 50,
      margin_bottom: 50,
      width: 1920 - 100,
      height: 50,
      ..Default::default()
    },
    ..Default::default()
  };
  backend::x11::run(&config.window);
}
