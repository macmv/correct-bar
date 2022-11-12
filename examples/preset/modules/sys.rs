use chrono::{Datelike, Timelike};
use correct_bar::bar::{Color, ModuleImpl, Updater};
use std::{cell::RefCell, time::Duration};
use sysinfo::{ComponentExt, CpuExt, SystemExt};

pub struct Time {
  pub primary:   Color,
  pub secondary: Color,
}

impl ModuleImpl for Time {
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
        )
      };
    }

    let local_rect = draw_time!(local);
    ctx.draw_text(" or ", self.secondary);
    let utc_rect = draw_time!(utc);

    fn copy_time<T: chrono::TimeZone>(time: chrono::DateTime<T>) {}

    if ctx.needs_click_regions() {
      ctx.add_on_click(local_rect, move || copy_time(chrono::Local::now()));
      ctx.add_on_click(utc_rect, move || copy_time(chrono::Utc::now()));
    }
  }
}

thread_local! {
  static SYS: RefCell<sysinfo::System> = RefCell::new(sysinfo::System::new_all());
}

pub struct Temp {
  pub primary:   Color,
  pub secondary: Color,
}
impl ModuleImpl for Temp {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    SYS.with(|s| {
      let mut sys = s.borrow_mut();
      sys.refresh_all();

      for c in sys.components() {
        if c.label() == "k10temp Tccd1" {
          ctx.draw_text(&format!("{:>2.00}", c.temperature()), self.primary);
          ctx.draw_text("°", self.secondary);
          break;
        }
      }
    });
  }
}

pub struct Cpu {
  pub primary:   Color,
  pub secondary: Color,
}
impl ModuleImpl for Cpu {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    SYS.with(|s| {
      let mut sys = s.borrow_mut();
      sys.refresh_all();

      ctx.draw_text(
        &format!(
          "{:>2.00}",
          sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32,
        ),
        self.primary,
      );
      ctx.draw_text("%", self.secondary);
    });
  }
}

pub struct Mem {
  pub primary:   Color,
  pub secondary: Color,
}
impl ModuleImpl for Mem {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    SYS.with(|s| {
      let mut sys = s.borrow_mut();
      sys.refresh_all();
      ctx.draw_text(
        &format!("{:>5.02}G", sys.used_memory() as f64 / (1024 * 1024 * 1024) as f64),
        self.primary,
      );
      ctx.draw_text(" / ", self.secondary);
      ctx.draw_text(
        &format!("{:>5.02}G", sys.total_memory() as f64 / (1024 * 1024 * 1024) as f64),
        self.primary,
      );
    });
  }
}
