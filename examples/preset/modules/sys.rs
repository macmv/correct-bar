use chrono::{Datelike, Timelike};
use correct_bar::bar::{Color, ModuleImpl, Updater};
use std::time::Duration;

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
