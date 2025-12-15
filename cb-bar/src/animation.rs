use std::cell::RefCell;

pub struct Animation {
  duration: f64,
  state:    RefCell<State>,
}

#[derive(Default)]
struct State {
  direction: Direction,
  time:      f64,
}

#[derive(Clone, Copy, Default, PartialEq)]
enum Direction {
  #[default]
  Start,
  Forward,
  Reverse,
  End,
}

impl Animation {
  pub fn linear(duration: f64) -> Animation { Animation { duration, state: Default::default() } }

  pub fn is_running(&self) -> bool { self.state.borrow().direction == Direction::Forward }

  pub fn interpolate(&self, start: f64, end: f64) -> f64 {
    let t = self.state.borrow().time / self.duration;
    start + (end - start) * t
  }

  pub fn start(&mut self) {
    let state = self.state.get_mut();
    state.time = 0.0;
    state.direction = Direction::Forward;
  }

  pub fn start_reverse(&mut self) {
    let state = self.state.get_mut();
    state.time = self.duration;
    state.direction = Direction::Reverse;
  }

  pub fn advance(&self, dt: std::time::Duration) {
    let mut state = self.state.borrow_mut();

    match state.direction {
      Direction::Start | Direction::End => {}
      Direction::Forward => {
        state.time += dt.as_secs_f64();

        if state.time >= self.duration {
          state.time = self.duration;
          state.direction = Direction::End;
        }
      }
      Direction::Reverse => {
        state.time -= dt.as_secs_f64();

        if state.time <= 0.0 {
          state.time = 0.0;
          state.direction = Direction::Start;
        }
      }
    }
  }
}
