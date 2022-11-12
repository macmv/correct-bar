pub mod desktop;
pub mod laptop;

use correct_bar::bar::{Color, Module, ModuleImpl, Updater};
use std::time::Duration;

struct TimeModule;

impl ModuleImpl for TimeModule {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let now = std::time::SystemTime::now();
    let dur = now.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
    ctx.draw_text(&format!("{}", dur.as_secs()), Color::white());
  }
}

pub fn modules() -> (Vec<Module>, Vec<Module>, Vec<Module>) {
  (
    vec![
      Module::text("foo", Color { r: 255, g: 255, b: 128 }).into(),
      Module::text(" | ", Color { r: 100, g: 100, b: 100 }).into(),
      Module::text("100%", Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("HELLO WORLD", Color { r: 255, g: 255, b: 128 }).into(),
      Module::text(" | ", Color { r: 100, g: 100, b: 100 }).into(),
      Module::text("foo and stuff", Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("mmm things", Color { r: 255, g: 100, b: 128 }).into(),
      Module::text(" | ", Color { r: 100, g: 100, b: 100 }).into(),
      Module::text("time here", Color { r: 100, g: 255, b: 128 }).into(),
      Module::from(TimeModule).into(),
    ],
  )
}
