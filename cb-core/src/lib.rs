use parley::{FontContext, LayoutContext};
use vello::Scene;

pub struct RenderStore {
  font:   FontContext,
  layout: LayoutContext,

  render: vello::Renderer,
}

pub struct Render<'a> {
  store: &'a mut RenderStore,
  scene: Scene,
}

impl Render<'_> {}
