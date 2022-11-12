pub mod desktop;
pub mod laptop;
pub mod modules;

use correct_bar::{
  bar::{Color, Module, ModuleImpl, Padding, Updater},
  math::Rect,
};

struct SepModule;

impl ModuleImpl for SepModule {
  fn padding_override(&self) -> Option<Padding> { Some(Padding::none()) }
  fn updater(&self) -> Updater { Updater::Never }
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    ctx.draw_rect(Rect { pos: ctx.pos(), width: 2, height: ctx.height() }, SEP);
    ctx.advance_by(2);
  }
}

const SEP: Color = Color::from_hex(0x888888);

pub fn modules() -> (Vec<Module>, Vec<Module>, Vec<Module>) {
  (
    vec![
      modules::BSPWM::new().into(),
      SepModule.into(),
      Module::text("100%", Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("HELLO WORLD", Color { r: 255, g: 255, b: 128 }).into(),
      SepModule.into(),
      Module::text("foo and stuff", Color { r: 100, g: 255, b: 128 }).into(),
    ],
    vec![
      Module::text("mmm things", Color { r: 255, g: 100, b: 128 }).into(),
      SepModule.into(),
      modules::Cpu::new().into(),
      SepModule.into(),
      modules::Mem::new().into(),
      SepModule.into(),
      modules::Time { primary: Color::white(), secondary: SEP }.into(),
    ],
  )
}
