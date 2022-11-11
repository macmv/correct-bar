mod backend;
pub mod bar;
pub mod config;
pub mod module;

use bar::Bar;
use config::Config;

pub fn run(config: Config) {
  let bar = backend::x11::setup(&config.window);
  loop {}
}
