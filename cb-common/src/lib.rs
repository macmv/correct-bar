use std::collections::HashMap;

pub struct Gpu {
  instance: wgpu::Instance,
  adapter:  wgpu::Adapter,
  device:   wgpu::Device,
  queue:    wgpu::Queue,

  bars: HashMap<BarId, Bar>,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct BarId(u32);

pub struct Bar {
  surface: wgpu::Surface<'static>,
}

impl BarId {
  pub fn new(id: u32) -> Self { BarId(id) }
}

impl Gpu {
  pub fn new() -> Self {
    let instance = wgpu::Instance::new(&Default::default());

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference:       wgpu::PowerPreference::default(),
      compatible_surface:     None,
      force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();

    Gpu { instance, adapter, device, queue, bars: HashMap::new() }
  }

  pub fn instance(&self) -> &wgpu::Instance { &self.instance }

  pub fn add_surface(&mut self, id: BarId, surface: wgpu::Surface, width: u32, height: u32) {
    let surface_caps = surface.get_capabilities(&self.adapter);
    // Shader code in this tutorial assumes an sRGB surface texture. Using a
    // different one will result in all the colors coming out darker. If you
    // want to support non sRGB surfaces, you'll need to account for that when
    // drawing to the frame.
    let surface_format =
      surface_caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width,
      height,
      present_mode: surface_caps.present_modes[0],
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };

    surface.configure(&self.device, &config);

    let output = surface.get_current_texture().unwrap();

    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

    {
      let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label:                    Some("Render Pass"),
        color_attachments:        &[Some(wgpu::RenderPassColorAttachment {
          view:           &view,
          resolve_target: None,
          depth_slice:    None,
          ops:            wgpu::Operations {
            load:  wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set:      None,
        timestamp_writes:         None,
      });
    }

    // submit will accept anything that implements IntoIter
    self.queue.submit(std::iter::once(encoder.finish()));
    output.present();
  }
}
