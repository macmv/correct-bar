use crossbeam_channel::Receiver;
use std::time::Duration;

pub struct Module {
  imp: Box<dyn ModuleImpl + Send + Sync>,
}

impl Module {
  pub fn imp(&self) -> &dyn ModuleImpl { &*self.imp }
  pub fn imp_mut(&mut self) -> &mut dyn ModuleImpl { &mut *self.imp }

  pub fn constant(sections: &[Section<'static>]) -> Module {
    struct ConstModule {
      sections: Vec<Section<'static>>,
    }
    impl ModuleImpl for ConstModule {
      fn render(&self) -> &[Section] { &self.sections }
      fn updater(&self) -> Updater { Updater::Never }
    }

    Module::from(ConstModule { sections: sections.into() })
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
  fn render(&self) -> &[Section];
  fn updater(&self) -> Updater;
}

#[derive(Clone)]
pub struct Section<'a> {
  pub text:      &'a str,
  pub color:     Option<u32>,
  pub clickable: bool,
}

impl<'a> Section<'a> {
  pub fn new(text: &'a str) -> Self { Section { text, color: None, clickable: false } }

  pub fn with_color(mut self, color: u32) -> Self {
    self.color = Some(color);
    self
  }
  pub fn with_clickable(mut self, clickable: bool) -> Self {
    self.clickable = clickable;
    self
  }
}
