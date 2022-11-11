use super::{Color, RenderContext};
use crossbeam_channel::Receiver;
use std::time::Duration;

pub struct Module {
  imp: Box<dyn ModuleImpl + Send + Sync>,
}

impl Module {
  pub fn imp(&self) -> &dyn ModuleImpl { &*self.imp }
  pub fn imp_mut(&mut self) -> &mut dyn ModuleImpl { &mut *self.imp }

  pub fn text(text: &'static str, color: Color) -> Module {
    struct TextModule {
      text:  &'static str,
      color: Color,
    }
    impl ModuleImpl for TextModule {
      fn render(&self, ctx: &mut RenderContext) { ctx.draw_text(self.text, self.color) }
      fn updater(&self) -> Updater { Updater::Never }
    }

    Module::from(TextModule { text, color })
  }
}

impl<T> From<T> for Module
where
  T: ModuleImpl + Send + Sync + 'static,
{
  fn from(imp: T) -> Self { Module { imp: Box::new(imp) } }
}

pub enum Updater {
  Never,
  Every(Duration),
  Channel(Receiver<()>),
}

pub trait ModuleImpl {
  fn render(&self, ctx: &mut RenderContext);
  fn updater(&self) -> Updater;
}
