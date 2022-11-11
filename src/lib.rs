mod backend;
pub mod bar;
pub mod config;
pub mod module;

use config::Config;

pub fn run(config: Config) {
  let bar = backend::x11::setup(&config.window);
  {
    let mut b = bar.lock();
    b.window_mut().draw_rect(10, 10, 20, 20, bar::Color { r: 100, g: 0, b: 200 });
    b.render();
  }
  loop {}
}
