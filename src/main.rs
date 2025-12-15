fn main() {
  cb_bar::run(cb_bar::Config {
    make_bar: || cb_bar::Bar {
      left_modules:   vec![],
      center_modules: vec![],
      right_modules:  vec![cb_builtin::Clock {
        primary:   peniko::Color::WHITE,
        secondary: peniko::Color::from_rgb8(0x80, 0x80, 0x80),
      }
      .into()],
    },
  });
}
