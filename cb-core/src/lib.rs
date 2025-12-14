use std::collections::HashMap;

use kurbo::{Affine, Point, Rect, Stroke, Vec2};
use parley::{FontContext, LayoutContext};
use peniko::{
  Brush, Color, Fill, Gradient,
  color::{AlphaColor, Oklch, OpaqueColor, Srgb},
};
use vello::{RenderParams, Scene};
use wgpu::util::TextureBlitter;

use crate::quad::Quad;

pub use cb_common::{App, BarId};
pub use wgpu;

mod quad;

pub struct RenderStore {
  font:   FontContext,
  layout: LayoutContext<peniko::Brush>,

  render: vello::Renderer,

  bars: HashMap<BarId, Bar>,
}

struct Bar {
  scale:        f32,
  blitter:      TextureBlitter,
  texture:      wgpu::Texture,
  texture_view: wgpu::TextureView,

  cursor: Option<Point>,
}

pub struct Render<'a> {
  bar:    BarId,
  cursor: Option<Point>,

  offset: Vec2,

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

    self.bars.insert(id, Bar { scale, texture, texture_view, blitter, cursor: None });
  }

  pub fn for_bar(&mut self, id: BarId) -> Option<Render<'_>> {
    if let Some(bar) = self.bars.get(&id) {
      Some(Render {
        bar:    id,
        cursor: bar.cursor,
        offset: Vec2::ZERO,
        store:  self,
        scene:  Scene::new(),
      })
    } else {
      None
    }
  }

  pub fn move_mouse(&mut self, id: BarId, pos: Option<(f64, f64)>) {
    self.bars.get_mut(&id).unwrap().cursor = pos.map(|(x, y)| Point::new(x as f64, y as f64));
  }

  pub fn set_scale(&mut self, id: BarId, device: &wgpu::Device, factor: i32) {
    let bar = self.bars.get_mut(&id).unwrap();
    bar.scale = factor as f32;

    bar.texture = device.create_texture(&wgpu::TextureDescriptor {
      label:           None,
      size:            wgpu::Extent3d {
        width:                 bar.texture.width() * factor as u32,
        height:                bar.texture.height() * factor as u32,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count:    1,
      dimension:       wgpu::TextureDimension::D2,
      usage:           wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
      format:          bar.texture.format(),
      view_formats:    &[],
    });
    bar.texture_view = bar.texture.create_view(&wgpu::TextureViewDescriptor::default());
  }
}

impl Render<'_> {
  pub fn set_offset(&mut self, offset: Vec2) { self.offset = offset; }

  fn transform(&self) -> Affine {
    let bar = &self.store.bars[&self.bar];
    Affine::scale(bar.scale.into()) * Affine::translate(self.offset)
  }

  pub fn draw_button(&mut self, shape: &impl kurbo::Shape, color: AlphaColor<Srgb>) {
    self.scene.stroke(&Stroke::new(2.0), self.transform(), &color, None, &shape);
  }

  pub fn draw_text(&mut self, origin: Point, text: &str, color: Color) -> Rect {
    let scale = self.store.bars[&self.bar].scale;

    let mut builder = self.store.layout.ranged_builder(&mut self.store.font, &text, 1.0, false);
    builder.push_default(parley::StyleProperty::Brush(color.into()));
    builder.push_default(parley::StyleProperty::FontSize(12.0 * scale));

    let mut layout: parley::Layout<peniko::Brush> = builder.build(&text);

    layout.break_all_lines(None);
    layout.align(None, parley::Alignment::Start, parley::AlignmentOptions::default());

    let mut rect = Rect::new(0.0, 0.0, f64::from(layout.width()), f64::from(layout.height()));

    for line in layout.lines() {
      for item in line.items() {
        let parley::PositionedLayoutItem::GlyphRun(glyph_run) = item else { continue };

        let run = glyph_run.run();
        rect.y0 = rect.y1.round() - rect.height();
        let mut x = rect.x0 as f32 + glyph_run.offset();
        let baseline = (rect.y0 as f32 + glyph_run.baseline()).round();

        self
          .scene
          .draw_glyphs(run.font())
          .brush(&glyph_run.style().brush)
          .hint(true)
          .transform(Affine::translate((origin.to_vec2() + self.offset) * f64::from(scale)))
          .glyph_transform(
            run.synthesis().skew().map(|angle| Affine::skew(angle.to_radians().tan() as f64, 0.0)),
          )
          .font_size(run.font_size())
          .normalized_coords(run.normalized_coords())
          .draw(
            Fill::NonZero,
            glyph_run.glyphs().map(|glyph| {
              let gx = x + glyph.x;
              let gy = baseline + glyph.y;
              x += glyph.advance;
              vello::Glyph { id: glyph.id.into(), x: gx, y: gy }
            }),
          );
      }
    }

    rect.scale_from_origin(1.0 / f64::from(scale)) + origin.to_vec2()
  }

  pub fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Texture) {
    let bar = &self.store.bars[&self.bar];

    fn oklch(l: f32, c: f32, h: f32) -> AlphaColor<Srgb> {
      OpaqueColor::<Oklch>::new([l, c, h]).to_rgba8().into()
    }

    let rect = kurbo::Rect::new(5.0, 5.0, 60.0, 28.0);

    let mut quad = Quad::from(rect);

    let brush = if let Some(cursor) = bar.cursor
      && rect.contains(cursor)
    {
      quad = Quad::new_tilted(rect, cursor, 12_f64.to_radians(), 100.0);

      let start = oklch(0.6, 0.1529, 259.41);
      let end = oklch(0.6, 0.1801, 283.76);
      Brush::Gradient(Gradient::new_linear(cursor, rect.center()).with_stops([start, end]))
    } else if let Some(cursor) = bar.cursor {
      let dx = (rect.x0 - cursor.x).max(cursor.x - rect.x1).max(0.0);
      let dy = (rect.y0 - cursor.y).max(cursor.y - rect.y1).max(0.0);
      let dist = (dx * dx + dy * dy).sqrt();

      if dist < 20.0 {
        let weight = (20.0 - dist) / 20.0;

        quad = Quad::new_tilted(rect, cursor, 12_f64.to_radians() * weight, 100.0);
      }

      oklch(0.6, 0.0, 0.0).into()
    } else {
      oklch(0.6, 0.0, 0.0).into()
    };

    self.scene.stroke(
      &Stroke::new(2.0),
      kurbo::Affine::scale(bar.scale.into()),
      &brush,
      None,
      &quad,
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
