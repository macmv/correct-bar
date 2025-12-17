use cb_bar::{Module, TextLayout};
use cb_core::{Color, Text};
use chrono::{Datelike, Timelike};

#[derive(Clone)]
pub struct Clock {
  pub primary:   Color,
  pub secondary: Color,
}

struct ClockModule {
  spec: Clock,
  text: Option<TextLayout>,
}

impl From<Clock> for Box<dyn Module> {
  fn from(spec: Clock) -> Self { Box::new(ClockModule { spec, text: None }) }
}

impl Module for ClockModule {
  fn updater(&self) -> cb_bar::Updater<'_> {
    cb_bar::Updater::Every(std::time::Duration::from_secs(1))
  }

  fn layout(&mut self, layout: &mut cb_bar::Layout) {
    layout.pad(5.0);

    let local = chrono::Local::now();
    let utc = local.naive_utc();

    let mut text = Text::new();
    text.push(&local.weekday().to_string(), Color::WHITE);
    text.push(", ", self.spec.secondary);
    text.push(
      &format_args!("{:04}-{:02}-{:02}", local.year(), local.month(), local.day()),
      Color::WHITE,
    );
    text.push(" at ", self.spec.secondary);

    macro_rules! draw_time {
      ( $time:expr ) => {
        text.push(
          &format_args!("{:02}:{:02}:{:02}", $time.hour(), $time.minute(), $time.second()),
          Color::WHITE,
        )
      };
    }

    draw_time!(local);
    text.push(" or ", self.spec.secondary);
    draw_time!(utc);

    self.text = Some(layout.layout_text(text, self.spec.secondary));

    layout.pad(5.0);
  }

  fn render(&self, ctx: &mut cb_core::Render) {
    if let Some(text) = &self.text {
      ctx.draw(text);
    }

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
