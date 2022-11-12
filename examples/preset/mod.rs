pub mod desktop;
pub mod laptop;

use chrono::{Datelike, Timelike};
use correct_bar::bar::{Color, Module, ModuleImpl, Updater};
use std::time::Duration;

struct TimeModule;

impl ModuleImpl for TimeModule {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let local = chrono::Local::now();
    let utc = local.naive_utc();
    ctx.draw_text(
      &format!(
        "{}, {:04}-{:02}-{:02} at {:02}:{:02}:{:02} or {:02}:{:02}:{:02}",
        local.weekday(),
        local.year(),
        local.month(),
        local.day(),
        local.hour(),
        local.minute(),
        local.second(),
        utc.hour(),
        utc.minute(),
        utc.second(),
      ),
      Color::white(),
    );
  }
}

fn sep() -> impl ModuleImpl { Module::text(" | ", Color { r: 100, g: 100, b: 100 }) }

pub fn modules() -> (Vec<Module>, Vec<Module>, Vec<Module>) {
  (
    vec![
      Module::text("foo", Color { r: 255, g: 255, b: 128 }).into(),
      sep().into(),
      Module::text("100%", Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("HELLO WORLD", Color { r: 255, g: 255, b: 128 }).into(),
      sep().into(),
      Module::text("foo and stuff", Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("mmm things", Color { r: 255, g: 100, b: 128 }).into(),
      sep().into(),
      Module::from(TimeModule).into(),
    ],
  )
}
