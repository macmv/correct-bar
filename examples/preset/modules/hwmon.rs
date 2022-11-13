//! Reads values in `/sys/class/hwmon` to get the temperature of various
//! devices.

use correct_bar::bar::{Color, ModuleImpl, Updater};
use std::{cell::RefCell, fs::File, time::Duration};

#[derive(Debug)]
struct Monitor {
  // The value of the `name` file.
  name: String,

  file: File,
}

impl Monitor {
  pub fn find_all() -> Vec<Monitor> {
    // TODO
    vec![]
  }
}

thread_local! {
  static MONITORS: RefCell<Option<Vec<Monitor>>> = RefCell::new(None);
}

pub struct Temp {
  pub primary:   Color,
  pub secondary: Color,
}
impl ModuleImpl for Temp {
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    MONITORS.with(|s| {
      let mut monitors = s.borrow_mut();
      if monitors.is_none() {
        *monitors = Some(Monitor::find_all());
      }
      dbg!(&monitors);

      let temp = 50.0;
      ctx.draw_text(&format!("{:>2.00}", temp), self.primary);
      ctx.draw_text("°", self.secondary);

      /*
      for c in state.components {
        if c.label() == "k10temp Tccd1" {
          ctx.draw_text(&format!("{:>2.00}", c.temperature()), self.primary);
          ctx.draw_text("°", self.secondary);
          break;
        }
      }
      */
    });
  }
}
