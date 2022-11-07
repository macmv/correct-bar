mod backend;
pub mod config;
pub mod module;

use config::Config;

pub fn run(config: Config) { backend::x11::run(&config.window); }
