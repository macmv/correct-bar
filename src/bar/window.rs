use super::{Color, Pos, Rect};

pub struct Window {
  data:   Vec<u8>,
  width:  u32,
  height: u32,
}

impl Window {
  pub fn new(width: u32, height: u32) -> Self {
    Window { data: vec![0; (width * height * 4) as usize], width, height }
  }
  pub fn width(&self) -> u32 { self.width }
  pub fn height(&self) -> u32 { self.height }
  pub fn data(&self) -> &[u8] { &self.data }

  pub fn draw_rect(&mut self, rect: Rect, color: Color) {
    for x in rect.left()..rect.right() {
      for y in rect.top()..rect.bottom() {
        self.draw_pixel(Pos { x, y }, color);
      }
    }
  }

  pub fn draw_pixel(&mut self, pos: Pos, color: Color) {
    // if i + 4 <= buf.len() {
    if pos.x < self.width && pos.y < self.height {
      let i = (pos.y * self.width + pos.x) as usize * 4;
      self.data[i] = color.r;
      self.data[i + 1] = color.g;
      self.data[i + 2] = color.b;
    }
  }
}
