use std::collections::HashMap;

use cb_common::BarId;
use kurbo::Stroke;
use parley::{FontContext, LayoutContext};
use peniko::{
  Brush, Color, Gradient,
  color::{AlphaColor, Oklch, OpaqueColor, Srgb},
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
  scale:        f32,
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
    scale: f32,
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

    let blitter = wgpu::util::TextureBlitterBuilder::new(&device, surface_format)
      .blend_state(wgpu::BlendState::ALPHA_BLENDING)
      .build();

    self.bars.insert(id, Bar { scale, texture, texture_view, blitter });
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

    fn oklch(l: f32, c: f32, h: f32) -> AlphaColor<Srgb> {
      OpaqueColor::<Oklch>::new([l, c, h]).to_rgba8().into()
    }

    let start = oklch(0.6, 0.1529, 259.41);
    let end = oklch(0.6, 0.1801, 283.76);
    let brush =
      Brush::Gradient(Gradient::new_linear((10.0, 5.0), (15.0, 15.0)).with_stops([start, end]));

    self.scene.stroke(
      &Stroke::new(5.0),
      kurbo::Affine::scale(bar.scale.into()),
      &brush,
      None,
      &kurbo::RoundedRect::new(5.0, 5.0, 60.0, 30.0, 8.0),
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
          base_color:          Color::from_rgba8(0, 0, 0, 0),
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
