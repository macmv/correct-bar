use correct_bar::{
  bar::{Color, Padding},
  config::{Config, WindowConfig},
};

pub fn run() {
  cb_bar::run(cb_bar::Config {
    bars: vec![cb_bar::Bar {
      left_modules:   vec![
        Box::new(cb_builtin::Clock {
          primary:   peniko::Color::from_rgb8(0x46, 0x46, 0x46),
          secondary: peniko::Color::from_rgb8(0x46, 0x46, 0x46),
        }),
        Box::new(cb_builtin::Clock {
          primary:   peniko::Color::from_rgb8(0x46, 0x46, 0x46),
          secondary: peniko::Color::from_rgb8(0x46, 0x46, 0x46),
        }),
      ],
      center_modules: vec![],
      right_modules:  vec![],
    }],
  });

  let (modules_left, modules_middle, modules_right) = super::modules(true);
  let config = Config {
    window: WindowConfig {
      height:         25,
      expected_width: 1920,
      margin_top:     0,
      margin_left:    0,
      margin_right:   0,
      margin_bottom:  0,
    },
    modules_left,
    modules_middle,
    modules_right,
    padding: Padding { left: 10, right: 10, top: 0, bottom: 0 },
    background: Color::from_hex(0x464646),
    font_size: 24.0,
    ..Default::default()
  };
  correct_bar::run(config)
}
