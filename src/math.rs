use std::ops::Add;

#[derive(Clone, Copy, Debug)]
pub struct Pos {
  pub x: u32,
  pub y: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
  pub pos:    Pos,
  pub width:  u32,
  pub height: u32,
}

impl Pos {
  pub fn within(&self, rect: Rect) -> bool {
    self.x >= rect.left()
      && self.y >= rect.top()
      && self.x <= rect.right()
      && self.y <= rect.bottom()
  }
}

impl Rect {
  pub fn left(&self) -> u32 { self.pos.x }
  pub fn right(&self) -> u32 { self.pos.x + self.width }
  pub fn top(&self) -> u32 { self.pos.y }
  pub fn bottom(&self) -> u32 { self.pos.y + self.height }

  pub fn with_width(mut self, width: u32) -> Self {
    self.width = width;
    self
  }
  pub fn with_height(mut self, height: u32) -> Self {
    self.height = height;
    self
  }

  /// Resizes the rectangle, and keeps the center where it is. This will move
  /// the position of the rectangle.
  pub fn resize_to(&mut self, width: u32, height: u32) {
    let center = self.center();
    self.pos.x = center.x - width / 2;
    self.pos.y = center.y - height / 2;
    self.width = width;
    self.height = height;
  }

  pub const fn center(&self) -> Pos {
    Pos { x: self.pos.x + self.width / 2, y: self.pos.y + self.height / 2 }
  }
}

impl Add for Pos {
  type Output = Self;

  #[track_caller]
  fn add(self, rhs: Self) -> Self::Output { Pos { x: self.x + rhs.x, y: self.y + rhs.y } }
}
