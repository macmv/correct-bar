use cb_core::{oklch, Color};

const GRAY: Color = Color::new([0.5, 0.0, 0.0, 1.0]);

fn main() {
  cb_bar::run(cb_bar::Config {
    make_bar: || cb_bar::Bar {
      left_modules:   vec![
        cb_builtin::Hypr { primary: oklch(0.7, 0.2, 310.0), secondary: GRAY }.into()
      ],
      center_modules: vec![],
      right_modules:  vec![
        cb_builtin::Temp { primary: oklch(0.7, 0.2, 310.0), secondary: GRAY }.into(),
        cb_builtin::Cpu { primary: oklch(0.7, 0.17, 20.0), secondary: GRAY }.into(),
        cb_builtin::Mem { primary: oklch(0.7, 0.19, 140.0), secondary: GRAY }.into(),
        cb_builtin::Clock { primary: Color::WHITE, secondary: GRAY }.into(),
      ],
    },
  });
}
