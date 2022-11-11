pub struct Bar {
  data:   Vec<u8>,
  width:  u32,
  height: u32,
  render: Box<dyn Fn(&[u8]) + Send + Sync>,
}

#[derive(Clone, Copy)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8,
}

impl Bar {
  pub fn new(width: u32, height: u32, render: impl Fn(&[u8]) + Send + Sync + 'static) -> Self {
    Bar {
      data: vec![0; width as usize * height as usize * 4],
      width,
      height,
      render: Box::new(render),
    }
  }

  pub fn width(&self) -> u32 { self.width }
  pub fn height(&self) -> u32 { self.height }

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

  pub fn render(&mut self) { (self.render)(&self.data); }
}
