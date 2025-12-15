use cb_core::{oklch, Color};

fn main() {
  cb_bar::run(cb_bar::Config {
    make_bar: || cb_bar::Bar {
      left_modules:   vec![],
      center_modules: vec![],
      right_modules:  vec![cb_builtin::Clock {
        primary:   Color::WHITE,
        secondary: oklch(0.5, 0.0, 0.0),
      }
      .into()],
    },
  });
}
