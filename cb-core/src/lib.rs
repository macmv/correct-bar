use std::collections::HashMap;

use cb_common::BarId;
use parley::{FontContext, LayoutContext};
use peniko::color::palette;
use vello::{RenderParams, Scene};

pub struct RenderStore {
  font:   FontContext,
  layout: LayoutContext,

  render: vello::Renderer,

  bars: HashMap<BarId, Bar>,
}

struct Bar {
  texture:      wgpu::Texture,
  texture_view: wgpu::TextureView,
}

pub struct Render<'a> {
  bar: BarId,

  store: &'a mut RenderStore,
  scene: Scene,
}

impl RenderStore {
  pub fn new(device: &wgpu::Device) -> Self {
    RenderStore {
      font:   FontContext::new(),
      layout: LayoutContext::new(),
      render: vello::Renderer::new(device, Default::default()).unwrap(),
      bars:   HashMap::new(),
    }
  }
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
        &self.store.bars[&self.bar].texture_view,
        &RenderParams {
          base_color:          palette::css::BLACK,
          width:               self.store.bars[&self.bar].texture.width(),
          height:              self.store.bars[&self.bar].texture.height(),
          antialiasing_method: vello::AaConfig::Msaa16,
        },
      )
      .unwrap();
  }
}
