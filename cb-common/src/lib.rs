pub struct Gpu {
  instance: wgpu::Instance,
}

impl Gpu {
  pub fn new() -> Self {
    let instance = wgpu::Instance::new(&Default::default());

    Gpu { instance }
  }

  pub fn instance(&self) -> &wgpu::Instance { &self.instance }

  pub fn add_surface(&mut self, surface: wgpu::Surface) {}
}
