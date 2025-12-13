use parley::{FontContext, LayoutContext};
use peniko::color::palette;
use vello::{RenderParams, Scene};

pub struct RenderStore {
  font:   FontContext,
  layout: LayoutContext,

  render:       vello::Renderer,
  texture:      wgpu::Texture,
  texture_view: wgpu::TextureView,
}

pub struct Render<'a> {
  store: &'a mut RenderStore,
  scene: Scene,
}

impl Render<'_> {
  pub fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    self
      .store
      .render
      .render_to_texture(
        device,
        queue,
        &self.scene,
        &self.store.texture_view,
        &RenderParams {
          base_color:          palette::css::BLACK,
          width:               self.store.texture.width(),
          height:              self.store.texture.height(),
          antialiasing_method: vello::AaConfig::Msaa16,
        },
      )
      .unwrap();
  }
}
