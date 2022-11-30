use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Pos {
  pub x: i32,
  pub y: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
  pub pos:    Pos,
  pub width:  u32,
  pub height: u32,
}

impl Pos {
  pub fn with_x(mut self, x: i32) -> Self {
    self.x = x;
    self
  }
  pub fn with_y(mut self, y: i32) -> Self {
    self.y = y;
    self
  }

  pub fn within(&self, rect: Rect) -> bool {
    self.x >= rect.left()
      && self.y >= rect.top()
      && self.x <= rect.right()
      && self.y <= rect.bottom()
  }

  pub fn scaled_by(mut self, scale: f64) -> Self {
    self.x = (self.x as f64 * scale) as i32;
    self.y = (self.y as f64 * scale) as i32;
    self
  }

  pub fn max(self, other: Pos) -> Pos { Pos { x: self.x.max(other.x), y: self.y.max(other.y) } }
  pub fn min(self, other: Pos) -> Pos { Pos { x: self.x.min(other.x), y: self.y.min(other.y) } }
}

impl Rect {
  pub fn left(&self) -> i32 { self.pos.x }
  pub fn right(&self) -> i32 { self.pos.x + self.width as i32 }
  pub fn top(&self) -> i32 { self.pos.y }
  pub fn bottom(&self) -> i32 { self.pos.y + self.height as i32 }

  pub fn with_x(mut self, x: i32) -> Self {
    self.pos.x = x;
    self
  }
  pub fn with_y(mut self, y: i32) -> Self {
    self.pos.y = y;
    self
  }
  pub fn with_width(mut self, width: u32) -> Self {
    self.width = width;
    self
  }
  pub fn with_height(mut self, height: u32) -> Self {
    self.height = height;
    self
  }

  pub fn scaled_by(mut self, scale: f64) -> Self {
    self.pos.x = (self.pos.x as f64 * scale) as i32;
    self.pos.y = (self.pos.y as f64 * scale) as i32;
    self.width = (self.width as f64 * scale) as u32;
    self.height = (self.height as f64 * scale) as u32;
    self
  }

  /// Resizes the rectangle, and keeps the center where it is. This will move
  /// the position of the rectangle.
  pub fn resize_to(&mut self, width: u32, height: u32) {
    let center = self.center();
    self.pos.x = center.x - width as i32 / 2;
    self.pos.y = center.y - height as i32 / 2;
    self.width = width;
    self.height = height;
  }

  pub const fn center(&self) -> Pos {
    Pos { x: self.pos.x + self.width as i32 / 2, y: self.pos.y + self.height as i32 / 2 }
  }
}

impl Add for Pos {
  type Output = Self;

  #[track_caller]
  fn add(self, rhs: Self) -> Self::Output { Pos { x: self.x + rhs.x, y: self.y + rhs.y } }
}

impl Sub for Pos {
  type Output = Self;

  #[track_caller]
  fn sub(self, rhs: Self) -> Self::Output { Pos { x: self.x - rhs.x, y: self.y - rhs.y } }
}

impl Mul<f64> for Pos {
  type Output = Self;

  #[track_caller]
  fn mul(self, rhs: f64) -> Self::Output {
    Pos { x: (self.x as f64 * rhs) as i32, y: (self.y as f64 * rhs) as i32 }
  }
}
