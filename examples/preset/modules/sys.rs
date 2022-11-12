use chrono::{Datelike, Timelike};
use correct_bar::bar::{Color, ModuleImpl, Updater};
use parking_lot::Mutex;
use std::time::Duration;
use sysinfo::{ComponentExt, CpuExt, CpuRefreshKind, RefreshKind, SystemExt};

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

pub struct Temp {
  sys: Mutex<sysinfo::System>,
}
impl Temp {
  pub fn new() -> Self {
    Temp {
      sys: Mutex::new(sysinfo::System::new_with_specifics(
        RefreshKind::new().with_components_list(),
      )),
    }
  }
}
impl ModuleImpl for Temp {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let mut sys = self.sys.lock();
    sys.refresh_all();

    for c in sys.components() {
      if c.label() == "k10temp Tccd1" {
        ctx.draw_text(&format!("{:>2.00}°", c.temperature()), Color::from_hex(0xff6600));
        break;
      }
    }
  }
}

pub struct Cpu {
  sys: Mutex<sysinfo::System>,
}
impl Cpu {
  pub fn new() -> Self {
    Cpu {
      sys: Mutex::new(sysinfo::System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
      )),
    }
  }
}
impl ModuleImpl for Cpu {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let mut sys = self.sys.lock();
    sys.refresh_all();

    ctx.draw_text(
      &format!(
        "{:>2.00}%",
        sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32,
      ),
      Color::from_hex(0xff0000),
    );
  }
}

pub struct Mem {
  pub primary:   Color,
  pub secondary: Color,
  sys:           Mutex<sysinfo::System>,
}
impl Mem {
  pub fn new(primary: Color, secondary: Color) -> Self {
    Mem {
      primary,
      secondary,
      sys: Mutex::new(sysinfo::System::new_with_specifics(RefreshKind::new().with_memory())),
    }
  }
}
impl ModuleImpl for Mem {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    let mut sys = self.sys.lock();
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
  }
}
