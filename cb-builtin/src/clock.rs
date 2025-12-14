use cb_bar::Module;
use kurbo::Point;
use peniko::Color;

#[derive(Clone)]
pub struct Clock {
  pub primary:   Color,
  pub secondary: Color,
}

impl Module for Clock {
  fn render(&self, ctx: &mut cb_core::Render) {
    let bounds = ctx.draw_text(Point::new(5.0, 8.0), "hello", self.secondary);

    ctx.draw_button(&bounds.inflate(3.0, 1.0), self.primary);

    /*
    let local = chrono::Local::now();
    let utc = local.naive_utc();

    ctx.draw_text(&local.weekday().to_string(), Color::WHITE);
    ctx.draw_text(", ", self.secondary);
    ctx.draw_text(
      &format!("{:04}-{:02}-{:02}", local.year(), local.month(), local.day()),
      Color::WHITE,
    );
    ctx.draw_text(" at ", self.secondary);

    macro_rules! draw_time {
      ( $time:expr ) => {
        ctx.draw_text(
          &format!("{:02}:{:02}:{:02}", $time.hour(), $time.minute(), $time.second()),
          Color::WHITE,
        )
      };
    }

    let local_rect = draw_time!(local);
    ctx.draw_text(" or ", self.secondary);
    let utc_rect = draw_time!(utc);
    */

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
