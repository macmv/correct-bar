use std::collections::HashMap;

use cb_common::BarId;
use kurbo::{Point, Shape, Size, Stroke, Vec2};
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

  cursor: Option<Point>,
}

pub struct Render<'a> {
  bar:    BarId,
  cursor: Option<Point>,

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
      Some(Render { bar: id, cursor: bar.cursor, store: self, scene: Scene::new() })
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

struct Quad {
  p: [Point; 4],
}

impl Render<'_> {
  pub fn draw(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, surface: &wgpu::Texture) {
    let bar = &self.store.bars[&self.bar];

    fn oklch(l: f32, c: f32, h: f32) -> AlphaColor<Srgb> {
      OpaqueColor::<Oklch>::new([l, c, h]).to_rgba8().into()
    }

    let rect = kurbo::Rect::new(5.0, 5.0, 60.0, 28.0);

    let mut quad = Quad {
      p: [
        Point::new(rect.x1, rect.y0),
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x0, rect.y1),
        Point::new(rect.x1, rect.y1),
      ],
    };

    let brush = if let Some(cursor) = bar.cursor
      && rect.inflate(5.0, 5.0).contains(cursor)
    {
      quad = tilted_button_quad(rect, cursor, 12_f64.to_radians(), 100.0);

      let start = oklch(0.6, 0.1529, 259.41);
      let end = oklch(0.6, 0.1801, 283.76);
      Brush::Gradient(Gradient::new_linear(cursor, rect.center()).with_stops([start, end]))
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

use nalgebra::{Point2, Rotation3, Vector2, Vector3};

fn tilted_button_quad(
  rect: kurbo::Rect,
  cursor_pos: Point,
  max_tilt_rad: f64,
  camera_dist: f64,
) -> Quad {
  let center = rect.center() - Vec2::new(rect.min_x(), rect.min_y());

  // Normalize cursor over button to [-1, 1]
  let nx = ((cursor_pos.x - rect.x0) / rect.width()) * 2.0 - 1.0;
  let ny = ((cursor_pos.y - rect.y0) / rect.height()) * 2.0 - 1.0;

  // Map to small rotations. Signs here are “feel” choices.
  let rot_y = nx * max_tilt_rad; // left/right tilt
  let rot_x = ny * max_tilt_rad; // up/down tilt

  let rot = Rotation3::from_euler_angles(rot_x, rot_y, 0.0);

  // Corners around center, in local 2D (y down). Convert to 3D with y up.
  let corners = [
    Vector3::new(-center.x, center.y, 0.0),  // top-left
    Vector3::new(center.x, center.y, 0.0),   // top-right
    Vector3::new(center.x, -center.y, 0.0),  // bottom-right
    Vector3::new(-center.x, -center.y, 0.0), // bottom-left
  ];

  let mut out = [Point::new(0.0, 0.0); 4];
  for (i, v) in corners.into_iter().enumerate() {
    // Rotate in 3D
    let r = rot * v;

    // Simple perspective: x' = x * d/(d - z), y' = y * d/(d - z)
    // (z toward camera makes it bigger)
    let denom = camera_dist - r.z;
    let s = camera_dist / denom;

    // Back to screen coords (y down) and translate to button center on screen
    let screen_cx = rect.min_x() + center.x;
    let screen_cy = rect.min_y() + center.y;

    out[i] = Point::new(screen_cx + r.x * s, screen_cy - r.y * s);
  }

  Quad { p: out }
}

impl kurbo::Shape for Quad {
  type PathElementsIter<'iter> = std::array::IntoIter<kurbo::PathEl, 5>;

  fn bounding_box(&self) -> kurbo::Rect {
    kurbo::Rect::new(
      self.p.iter().map(|p| p.x).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
      self.p.iter().map(|p| p.y).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
      self.p.iter().map(|p| p.x).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
      self.p.iter().map(|p| p.y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap(),
    )
  }
  fn winding(&self, _pt: Point) -> i32 { 0 }
  fn perimeter(&self, _accuracy: f64) -> f64 { 0.0 }
  fn area(&self) -> f64 { 0.0 }
  fn to_path(&self, tolerance: f64) -> kurbo::BezPath {
    kurbo::BezPath::from_iter(self.path_elements(tolerance))
  }

  fn path_elements(&self, _tolerance: f64) -> Self::PathElementsIter<'_> {
    [
      kurbo::PathEl::MoveTo(self.p[0]),
      kurbo::PathEl::LineTo(self.p[1]),
      kurbo::PathEl::LineTo(self.p[2]),
      kurbo::PathEl::LineTo(self.p[3]),
      kurbo::PathEl::ClosePath,
    ]
    .into_iter()
  }
}
