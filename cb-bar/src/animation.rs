pub struct Animation {
  time:     f64,
  duration: f64,
  state:    State,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
  Start,
  Running,
  Done,
}

impl Animation {
  pub fn linear(duration: f64) -> Animation {
    Animation { time: 0.0, duration, state: State::Start }
  }

  pub fn is_running(&self) -> bool { self.state == State::Running }

  pub fn start(&mut self) {
    self.time = 0.0;
    self.state = State::Running;
  }

  pub fn advance(&mut self, dt: std::time::Duration) {
    if self.state != State::Running {
      return;
    }

    self.time += dt.as_secs_f64();

    if self.time >= self.duration {
      self.state = State::Done;
    }
  }
}
