use correct_bar::bar::{Color, ModuleImpl, Updater};
use std::{
  cell::RefCell,
  time::{Duration, Instant},
};
use sysinfo::{ComponentExt, CpuExt, SystemExt};

thread_local! {
  static SYS: RefCell<SystemInfo> = RefCell::new(SystemInfo {
    last_update: None,
    info:        sysinfo::System::new_all()
  });
}

struct SystemInfo {
  last_update: Option<Instant>,
  info:        sysinfo::System,
}

impl SystemInfo {
  fn refresh(&mut self) {
    if let Some(last_update) = self.last_update {
      if last_update.elapsed() > Duration::from_secs(1) {
        self.update();
      }
    } else {
      self.update();
    }
  }

  fn update(&mut self) { self.info.refresh_all(); }
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
      sys.refresh();

      for c in sys.info.components() {
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
      sys.refresh();

      ctx.draw_text(
        &format!(
          "{:>2.00}",
          sys.info.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.info.cpus().len() as f32,
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
      sys.refresh();
      ctx.draw_text(
        &format!("{:>5.02}G", sys.info.used_memory() as f64 / (1024 * 1024 * 1024) as f64),
        self.primary,
      );
      ctx.draw_text(" / ", self.secondary);
      ctx.draw_text(
        &format!("{:>5.02}G", sys.info.total_memory() as f64 / (1024 * 1024 * 1024) as f64),
        self.primary,
      );
    });
  }
}
