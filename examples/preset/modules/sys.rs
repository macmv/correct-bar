use chrono::{Datelike, Timelike};
use correct_bar::{
  bar::{Color, ModuleImpl, Updater},
  math::Rect,
};
use parking_lot::Mutex;
use std::time::Duration;
use sysinfo::{CpuExt, SystemExt};

pub struct Time {
  pub primary:   Color,
  pub secondary: Color,
}

impl ModuleImpl for Time {
  fn background(&self) -> Option<Color> { Some(Color::from_hex(0x001122)) }
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let local = chrono::Local::now();
    let utc = local.naive_utc();

    ctx.draw_text(&local.weekday().to_string(), Color::white());
    ctx.draw_text(", ", self.secondary);
    ctx.draw_text(
      &format!("{:04}-{:02}-{:02}", local.year(), local.month(), local.day()),
      Color::white(),
    );
    ctx.draw_text(" at ", self.secondary);

    macro_rules! draw_time {
      ( $time:expr ) => {
        ctx.draw_text(
          &format!("{:02}:{:02}:{:02}", $time.hour(), $time.minute(), $time.second()),
          Color::white(),
        );
      };
    }

    draw_time!(local);
    ctx.draw_text(" or ", self.secondary);
    draw_time!(utc);
  }
}

pub struct CpuMem {
  sys: Mutex<sysinfo::System>,
}

impl CpuMem {
  pub fn new() -> Self { CpuMem { sys: Mutex::new(sysinfo::System::new_all()) } }
}

impl ModuleImpl for CpuMem {
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
    ctx.draw_rect(Rect { pos: ctx.pos(), width: 2, height: ctx.height() }, Color::white());
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
