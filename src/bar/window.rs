use super::Color;

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

  pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
    for x in x..x + width {
      for y in y..y + height {
        self.draw_pixel(x, y, color);
      }
    }
  }

  pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
    // if i + 4 <= buf.len() {
    if x < self.width && y < self.height {
      let i = (x * self.width + y) as usize * 4;
      self.data[i] = color.r;
      self.data[i + 1] = color.g;
      self.data[i + 2] = color.b;
    }
  }
}
