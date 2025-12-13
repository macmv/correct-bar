use std::collections::HashMap;

use cb_common::BarId;
use parley::{FontContext, LayoutContext};
use peniko::color::palette;
use vello::{RenderParams, Scene, util::RenderSurface};

pub struct RenderStore {
  font:   FontContext,
  layout: LayoutContext,

  render: vello::Renderer,

  bars: HashMap<BarId, Bar>,
}

struct Bar {
  surface: RenderSurface<'static>,

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

  pub fn for_bar(&mut self, id: BarId) -> Option<Render<'_>> {
    if self.bars.contains_key(&id) {
      Some(Render { bar: id, store: self, scene: Scene::new() })
    } else {
      None
    }
  }
}

impl Render<'_> {
  pub fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Texture) {
    let bar = &self.store.bars[&self.bar];

    self
      .store
      .render
      .render_to_texture(
        device,
        queue,
        &self.scene,
        &bar.texture_view,
        &RenderParams {
          base_color:          palette::css::BLACK,
          width:               bar.texture.width(),
          height:              bar.texture.height(),
          antialiasing_method: vello::AaConfig::Msaa16,
        },
      )
      .unwrap();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    bar.surface.blitter.copy(
      device,
      &mut encoder,
      &bar.texture_view,
      &surface.create_view(&wgpu::TextureViewDescriptor::default()),
    );

    // submit will accept anything that implements IntoIter
    queue.submit(std::iter::once(encoder.finish()));
  }
}
