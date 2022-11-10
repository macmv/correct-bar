use std::cell::RefCell;

pub struct Bar {
  data:   RefCell<Vec<u8>>,
  width:  u32,
  height: u32,
}

#[derive(Clone, Copy)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8,
}

impl Bar {
  pub fn new(width: u32, height: u32) -> Self {
    Bar { data: RefCell::new(vec![0; width as usize * height as usize * 4]), width, height }
  }

  pub fn width(&self) -> u32 { self.width }
  pub fn height(&self) -> u32 { self.height }

  pub fn draw_rect(&self, x: u32, y: u32, width: u32, height: u32, color: Color) {
    for x in x..x + width {
      for y in y..y + height {
        self.draw_pixel(x, y, color);
      }
    }
  }

  pub fn draw_pixel(&self, x: u32, y: u32, color: Color) {
    // if i + 4 <= buf.len() {
    if x < self.width && y < self.height {
      let i = (x * self.width + y) as usize * 4;
      let mut buf = self.data.borrow_mut();
      buf[i] = color.r;
      buf[i + 1] = color.g;
      buf[i + 2] = color.b;
    }
  }
}
