mod backend;
mod config;

pub use config::Config;

pub fn run() {
  let config = Config {
    window: config::WindowConfig {
      margin_top: 40,
      margin_left: 10,
      margin_right: 10,
      margin_bottom: 10,
      width: 1920 - 20,
      height: 50,
      ..Default::default()
    },
    ..Default::default()
  };
  backend::x11::run(&config.window);
}
