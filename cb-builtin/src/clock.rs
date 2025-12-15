use cb_bar::Module;
use cb_core::Text;
use chrono::{Datelike, Timelike};
use kurbo::Point;
use peniko::Color;

#[derive(Clone)]
pub struct Clock {
  pub primary:   Color,
  pub secondary: Color,
}

struct ClockModule {
  spec: Clock,
}

impl From<Clock> for Box<dyn Module> {
  fn from(spec: Clock) -> Self { Box::new(ClockModule { spec }) }
}

impl Module for ClockModule {
  fn updater(&self) -> cb_bar::Updater { cb_bar::Updater::Every(std::time::Duration::from_secs(1)) }

  fn render(&self, ctx: &mut cb_core::Render) {
    let local = chrono::Local::now();
    let utc = local.naive_utc();

    let mut text = Text::new();
    text.push(&local.weekday().to_string(), Color::WHITE.convert());
    text.push(", ", self.spec.secondary.convert());
    text.push(
      &format_args!("{:04}-{:02}-{:02}", local.year(), local.month(), local.day()),
      Color::WHITE.convert(),
    );
    text.push(" at ", self.spec.secondary.convert());

    macro_rules! draw_time {
      ( $time:expr ) => {
        text.push(
          &format_args!("{:02}:{:02}:{:02}", $time.hour(), $time.minute(), $time.second()),
          Color::WHITE.convert(),
        )
      };
    }

    draw_time!(local);
    text.push(" or ", self.spec.secondary.convert());
    draw_time!(utc);

    ctx.draw_text(Point::new(5.0, 8.0), text, self.spec.secondary.convert());

    /*
    fn copy(str: String) {
      use std::{
        io::Write,
        process::{Command, Stdio},
      };
      Command::new("notify-send").arg(format!("Copying {str}")).output().unwrap();

      let mut child =
        Command::new("xclip").stdin(Stdio::piped()).arg("-sel").arg("clip").spawn().unwrap();
      let mut stdin = child.stdin.take().unwrap();
      stdin.write_all(str.as_bytes()).unwrap();
      // Close stdin before waiting
      drop(stdin);
      child.wait().unwrap();
    }

    if ctx.needs_click_regions() {
      ctx.add_on_click(local_rect, move || {
        let now = chrono::Local::now();
        copy(format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second()));
      });
      ctx.add_on_click(utc_rect, move || {
        let now = chrono::Local::now();
        copy(now.timestamp().to_string())
      });
    }
    */
  }
}
