fn main() {
  cb_bar::run(cb_bar::Config {
    bars: vec![cb_bar::Bar {
      left_modules:   vec![Box::new(cb_builtin::Clock {
        primary:   peniko::Color::from_rgb8(0x46, 0x46, 0x46),
        secondary: peniko::Color::from_rgb8(0x46, 0x46, 0x46),
      })],
      center_modules: vec![],
      right_modules:  vec![Box::new(cb_builtin::Clock {
        primary:   peniko::Color::from_rgb8(0x46, 0x46, 0x46),
        secondary: peniko::Color::from_rgb8(0x46, 0x46, 0x46),
      })],
    }],
  });
}
