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

impl Rect {
  pub fn left(&self) -> u32 { self.pos.x }
  pub fn right(&self) -> u32 { self.pos.x + self.width }
  pub fn top(&self) -> u32 { self.pos.y }
  pub fn bottom(&self) -> u32 { self.pos.y + self.height }
}

impl Add for Pos {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output { Pos { x: self.x + rhs.x, y: self.y + rhs.y } }
}
