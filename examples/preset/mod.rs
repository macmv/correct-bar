pub mod desktop;
pub mod laptop;

use chrono::{Datelike, Timelike};
use correct_bar::bar::{Color, Module, ModuleImpl, Padding, Rect, Updater};
use parking_lot::Mutex;
use std::time::Duration;
use sysinfo::{CpuExt, SystemExt};

struct SepModule;

impl ModuleImpl for SepModule {
  fn padding_override(&self) -> Option<Padding> { Some(Padding::none()) }
  fn updater(&self) -> Updater { Updater::Never }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    ctx.draw_rect(Rect { pos: ctx.pos(), width: 2, height: ctx.height() }, SEP);
    ctx.advance_by(2);
  }
}

struct TimeModule;

impl ModuleImpl for TimeModule {
  fn background(&self) -> Option<Color> { Some(Color::from_hex(0x001122)) }
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let local = chrono::Local::now();
    let utc = local.naive_utc();

    ctx.draw_text(&local.weekday().to_string(), Color::white());
    ctx.draw_text(", ", SEP);
    ctx.draw_text(
      &format!("{:04}-{:02}-{:02}", local.year(), local.month(), local.day()),
      Color::white(),
    );
    ctx.draw_text(" at ", SEP);

    macro_rules! draw_time {
      ( $time:expr ) => {
        ctx.draw_text(
          &format!("{:02}:{:02}:{:02}", $time.hour(), $time.minute(), $time.second()),
          Color::white(),
        );
      };
    }

    draw_time!(local);
    ctx.draw_text(" or ", SEP);
    draw_time!(utc);
  }
}

struct CpuMemModule {
  sys: Mutex<sysinfo::System>,
}

impl CpuMemModule {
  pub fn new() -> Self { CpuMemModule { sys: Mutex::new(sysinfo::System::new_all()) } }
}

impl ModuleImpl for CpuMemModule {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let mut sys = self.sys.lock();
    sys.refresh_all();
    ctx.draw_text(
      &format!(
        "{:>5.02} / {:>5.02}",
        sys.used_memory() as f64 / (1024 * 1024 * 1024) as f64,
        sys.total_memory() as f64 / (1024 * 1024 * 1024) as f64,
      ),
      Color::from_hex(0xffff00),
    );
    ctx.advance_by(ctx.padding().left);
    ctx.draw_rect(Rect { pos: ctx.pos(), width: 2, height: ctx.height() }, SEP);
    ctx.advance_by(ctx.padding().right + 2);
    ctx.draw_text(
      &format!(
        "{:>2.00}%",
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32,
      ),
      Color::from_hex(0xff0000),
    );
  }
}

const SEP: Color = Color::from_hex(0x888888);

pub fn modules() -> (Vec<Module>, Vec<Module>, Vec<Module>) {
  (
    vec![
      Module::text("foo", Color { r: 255, g: 255, b: 128 }).into(),
      SepModule.into(),
      Module::text("100%", Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("HELLO WORLD", Color { r: 255, g: 255, b: 128 }).into(),
      SepModule.into(),
      Module::text("foo and stuff", Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("mmm things", Color { r: 255, g: 100, b: 128 }).into(),
      SepModule.into(),
      CpuMemModule::new().into(),
      SepModule.into(),
      TimeModule.into(),
    ],
  )
}
