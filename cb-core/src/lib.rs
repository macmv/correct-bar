use std::collections::HashMap;

use cb_common::BarId;
use parley::{FontContext, LayoutContext};
use peniko::{
  Color,
  color::{AlphaColor, Oklch, OpaqueColor, palette},
};
use vello::{RenderParams, Scene};
use wgpu::util::TextureBlitter;

pub struct RenderStore {
  font:   FontContext,
  layout: LayoutContext,

  render: vello::Renderer,

  bars: HashMap<BarId, Bar>,
}

struct Bar {
  blitter:      TextureBlitter,
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

  pub fn create_bar(
    &mut self,
    id: BarId,
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
  ) {
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label: None,
      size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
      format,
      view_formats: &[],
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let blitter = TextureBlitter::new(&device, surface_format);

    self.bars.insert(id, Bar { texture, texture_view, blitter });
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

    self.scene.fill(
      peniko::Fill::NonZero,
      kurbo::Affine::IDENTITY,
      Color::from_rgb8(55, 55, 55),
      None,
      &kurbo::Rect::new(5.0, 5.0, 15.0, 15.0),
    );

    self
      .store
      .render
      .render_to_texture(
        device,
        queue,
        &self.scene,
        &bar.texture_view,
        &RenderParams {
          base_color:          /* OpaqueColor::<Oklch>::new([0.33, 0.0, 292.24]).to_rgba8().into() */ Color::from_rgb8(128, 128, 128),
          width:               bar.texture.width(),
          height:              bar.texture.height(),
          antialiasing_method: vello::AaConfig::Msaa16,
        },
      )
      .unwrap();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    bar.blitter.copy(
      device,
      &mut encoder,
      &bar.texture_view,
      &surface.create_view(&wgpu::TextureViewDescriptor::default()),
    );

    // submit will accept anything that implements IntoIter
    queue.submit(std::iter::once(encoder.finish()));
  }
}
