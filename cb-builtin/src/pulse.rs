use cb_bar::{Module, Updater};
use cb_core::{Color, Render};
use crossbeam_channel::Receiver;
use libpulse_sys as sys;
use parking_lot::Mutex;
use std::{
  ffi::{CStr, CString, c_void},
  fmt,
  mem::ManuallyDrop,
  ptr,
  sync::Arc,
};

pub struct Pulse {
  pub color: Color,
}

struct PulseModule {
  spec: Pulse,
}

struct MainLoop {
  pa: *mut sys::pa_mainloop,
}

unsafe impl Send for MainLoop {}

impl Drop for MainLoop {
  fn drop(&mut self) {
    unsafe {
      sys::pa_mainloop_free(self.pa);
    }
  }
}

impl MainLoop {
  pub fn new() -> Self {
    unsafe {
      let pa = sys::pa_mainloop_new();

      MainLoop { pa }
    }
  }

  pub fn run(&mut self) {
    unsafe {
      sys::pa_mainloop_run(self.pa, ptr::null_mut());
    }
  }
}

struct Context {
  pa: *mut sys::pa_context,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe {
      sys::pa_context_unref(self.pa);
    }
  }
}

impl Clone for Context {
  fn clone(&self) -> Self {
    unsafe {
      sys::pa_context_ref(self.pa);
    }
    Context { pa: self.pa }
  }
}

macro_rules! callback {
  ($name:ident($($arg_name:ident: $arg_ty:ty)*), $sys:ident, dyn FnOnce($ty:ty), |$info:ident: $info_ty:ty| $constructor:expr) => {
    #[allow(unused)]
    pub fn $name(&self, $($arg_name: $arg_ty,)* custom: impl FnOnce($ty) + Send + 'static) {
      extern "C" fn callback(
        _ctx: *mut sys::pa_context,
        $info: $info_ty,
        ptr: *mut c_void
      ) {
        unsafe {
          let cb = Box::from_raw(ptr.cast::<Box<dyn FnOnce($ty)>>());
          cb($constructor);
        }
      }

      unsafe {
        // Box it up twice:
        // - The outer box is converted to a pointer and passed through pa_context.
        // - The inner box is a fat pointer to allow for a `dyn` fn.
        let cb: Box<Box<dyn FnOnce($ty) + Send>> = Box::new(Box::new(custom));
        sys::$sys(
          self.pa,
          $($arg_name,)*
          Some(callback),
          Box::into_raw(cb).cast::<c_void>(),
        );
      }
    }
  };
}

macro_rules! callback_list {
  ($name:ident($($arg_name:ident: $arg_ty:ty)*), $sys:ident, dyn FnMut($ty:ty), |$info:ident: $info_ty:ty| $constructor:expr) => {
    #[allow(unused)]
    pub fn $name(&self, $($arg_name: $arg_ty,)* custom: impl FnMut($ty) + Send + 'static) {
      extern "C" fn callback(
        _ctx: *mut sys::pa_context,
        $info: $info_ty,
        eol: i32,
        ptr: *mut c_void
      ) {
        unsafe {
          let cb = Box::from_raw(ptr.cast::<Box<dyn FnMut($ty)>>());
          if eol == 0 {
            // Make sure to keep this box around.
            let cb = Box::leak(cb);
            cb($constructor);
          } else {
            // Now that `eol` is nonzero, we're at the end of the list, so we drop the `cb`.
            drop(cb);
          }
        }
      }

      unsafe {
        // Box it up twice:
        // - The outer box is converted to a pointer and passed through pa_context.
        // - The inner box is a fat pointer to allow for a `dyn` fn.
        let cb: Box<Box<dyn FnMut($ty) + Send>> = Box::new(Box::new(custom));
        sys::$sys(
          self.pa,
          $($arg_name,)*
          Some(callback),
          Box::into_raw(cb).cast::<c_void>(),
        );
      }
    }
  };
}

impl Context {
  pub fn new(mainloop: &mut MainLoop, props: &PropList) -> Self {
    unsafe {
      let pa = sys::pa_context_new_with_proplist(
        sys::pa_mainloop_get_api(mainloop.pa),
        b"correct-bar\0".as_ptr() as *const _,
        props.pa,
      );

      Context { pa }
    }
  }

  pub fn set_callback(&mut self, custom: impl Fn() + Send + 'static) {
    static CALLBACK: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
    extern "C" fn cb(_ctx: *mut sys::pa_context, _userdata: *mut c_void) {
      let cb = CALLBACK.lock();
      if let Some(cb) = &*cb {
        cb();
      }
    }

    if CALLBACK.lock().replace(Box::new(custom)).is_some() {
      panic!("callback already set");
    }
    unsafe {
      sys::pa_context_set_state_callback(self.pa, Some(cb), ptr::null_mut());
    }
  }

  pub fn set_on_change(&self, callback: impl Fn() + Send + 'static) {
    extern "C" fn cb(
      _ctx: *mut sys::pa_context,
      _ev: sys::pa_subscription_event_type_t,
      _idx: u32,
      ptr: *mut c_void,
    ) {
      unsafe {
        let cb = &*ptr.cast::<Box<dyn Fn() + Send + 'static>>();
        cb();
      }
    }

    unsafe {
      let ptr: &mut Box<dyn Fn() + Send + 'static> = Box::leak(Box::new(Box::new(callback)));
      sys::pa_context_set_subscribe_callback(
        self.pa,
        Some(cb),
        std::ptr::from_mut(ptr) as *mut c_void,
      );
      sys::pa_context_subscribe(self.pa, sys::PA_SUBSCRIPTION_MASK_SINK, None, ptr::null_mut());
    }
  }

  pub fn connect(&mut self) {
    unsafe {
      sys::pa_context_connect(
        self.pa,
        std::ptr::null(),
        sys::PA_CONTEXT_NOAUTOSPAWN,
        std::ptr::null(),
      );
    }
  }

  pub fn get_state(&self) -> ContextState {
    unsafe { ContextState::from_sys(sys::pa_context_get_state(self.pa)) }
  }

  callback!(
    get_server_info(),
    pa_context_get_server_info,
    dyn FnOnce(&ServerInfo),
    |info: *const sys::pa_server_info| &ServerInfo { pa: info }
  );

  callback_list!(
    get_sink_info_list(),
    pa_context_get_sink_info_list,
    dyn FnMut(SinkInfo),
    |info: *const sys::pa_sink_info| SinkInfo { pa: info }
  );
  callback_list!(
    get_source_info_list(),
    pa_context_get_source_info_list,
    dyn FnMut(SourceInfo),
    |info: *const sys::pa_source_info| SourceInfo { pa: info }
  );

  callback_list!(
    get_sink_input_info_list(),
    pa_context_get_sink_input_info_list,
    dyn FnMut(SinkInputInfo),
    |info: *const sys::pa_sink_input_info| SinkInputInfo { pa: info }
  );
  callback_list!(
    get_source_output_info_list(),
    pa_context_get_source_output_info_list,
    dyn FnMut(SourceOutputInfo),
    |info: *const sys::pa_source_output_info| SourceOutputInfo { pa: info }
  );
}

struct PropList {
  pa: *mut sys::pa_proplist,
}

impl Drop for PropList {
  fn drop(&mut self) {
    unsafe {
      sys::pa_proplist_free(self.pa);
    }
  }
}

impl PropList {
  pub fn new() -> Self {
    unsafe {
      let pa = sys::pa_proplist_new();

      PropList { pa }
    }
  }

  pub fn set(&mut self, k: &str, v: &str) {
    unsafe {
      let k = CString::new(k).unwrap();
      let v = CString::new(v).unwrap();

      sys::pa_proplist_sets(self.pa, k.as_ptr(), v.as_ptr());
    }
  }
}

impl fmt::Debug for PropList {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    unsafe {
      let str = CStr::from_ptr(sys::pa_proplist_to_string(self.pa));

      write!(f, "{}", str.to_string_lossy())?;

      sys::pa_xfree(str.as_ptr() as *mut c_void);
    }
    Ok(())
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ContextState {
  Unconnected,
  Connecting,
  Authorizing,
  SettingName,
  Ready,
  Failed,
  Terminated,
  Unknown,
}

impl ContextState {
  pub fn from_sys(s: sys::pa_context_state_t) -> Self {
    match s {
      sys::PA_CONTEXT_UNCONNECTED => ContextState::Unconnected,
      sys::PA_CONTEXT_CONNECTING => ContextState::Connecting,
      sys::PA_CONTEXT_AUTHORIZING => ContextState::Authorizing,
      sys::PA_CONTEXT_SETTING_NAME => ContextState::SettingName,
      sys::PA_CONTEXT_READY => ContextState::Ready,
      sys::PA_CONTEXT_FAILED => ContextState::Failed,
      sys::PA_CONTEXT_TERMINATED => ContextState::Terminated,

      #[allow(unreachable_patterns)]
      _ => ContextState::Unknown,
    }
  }
}

macro_rules! info_getter {
  ($self:ident, $field:ident: &str) => {
    unsafe { CStr::from_ptr((*$self.pa).$field).to_str().unwrap() }
  };
  ($self:ident, $field:ident: Volume) => {
    unsafe { Volume { pa: (*$self.pa).$field } }
  };
  ($self:ident, $field:ident: $ty:ty) => {
    unsafe { (*$self.pa).$field }
  };
}

macro_rules! info {
  { $name: ident =>
    $(
      $(#[$meta:meta])* $field:ident($($ty:tt)*);
    )*
  } => {
    impl $name {
      $(
        $(#[$meta])*
        pub fn $field(&self) -> $($ty)* { info_getter!(self, $field: $($ty)*) }
      )*
    }

    impl fmt::Debug for $name {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!($name))
          $(
            .field(stringify!($field), &self.$field())
          )*
          .finish()
      }
    }
  };
}

struct ServerInfo {
  pa: *const sys::pa_server_info,
}

info! { ServerInfo =>
  /// User name of the daemon process.
  user_name(&str);
  /// Host name the daemon is running on.
  host_name(&str);
  /// Version string of the daemon.
  server_version(&str);
  /// Server package name (usually "pulseaudio").
  server_name(&str);
  /// Default sample specification
  sample_spec(sys::pa_sample_spec);
  /// Name of default sink.
  default_sink_name(&str);
  /// Name of default source.
  default_source_name(&str);
  /// A random cookie for identifying this instance of PulseAudio.
  cookie(u32);
  /// Default channel map.
  channel_map(sys::pa_channel_map);
}

struct SourceOutputInfo {
  pa: *const sys::pa_source_output_info,
}

info! { SourceOutputInfo =>
  /// Index of the source output.
  index(u32);
  /// Name of the source output.
  name(&str);
  /// Index of the module this source output belongs to, or PA_INVALID_INDEX when it does not belong to any module.
  owner_module(u32);
  /// Index of the client this source output belongs to, or PA_INVALID_INDEX when it does not belong to any client.
  client(u32);
  /// Index of the connected source.
  source(u32);
  // /// The sample specification of the source output.
  // sample_spec( pa_sample_spec);
  // /// Channel map.
  // channel_map( pa_channel_map);
  // /// Latency due to buffering in the source output, see pa_timing_info for details.
  // buffer_usec( pa_usec_t);
  // /// Latency of the source device, see pa_timing_info for details.
  // source_usec( pa_usec_t);
  /// The resampling method used by this source output.
  resample_method(&str);
  /// Driver name.
  driver(&str);
  // /// Property list.
  // proplist(pa_proplist);
  /// Stream corked.
  corked(i32);
  // /// The volume of this source output.
  // volume( pa_cvolume);
  /// Stream muted.
  mute(i32);
  /// Stream has volume. If not set, then the meaning of this struct's volume member is unspecified.
  has_volume(i32);
  /// The volume can be set. If not set, the volume can still change even though clients can't control the volume.
  volume_writable(i32);
  // /// Stream format information.
  // format( pa_format_info);
}

struct SinkInputInfo {
  pa: *const sys::pa_sink_input_info,
}

info! { SinkInputInfo =>
  /// Index of the sink input
  index(u32);
  /// Name of the sink input
  name(&str);
  /// Index of the module this sink input belongs to, or PA_INVALID_INDEX when it does not belong to any module.
  owner_module(u32);
  /// Index of the client this sink input belongs to, or PA_INVALID_INDEX when it does not belong to any client.
  client(u32);
  /// Index of the connected sink
  sink(u32);

  // /// The sample specification of the sink input.
  // pa_sample_spec sample_spec;
  // /// Channel map
  // pa_channel_map channel_map;
  // /// The volume of this sink input.
  // pa_cvolume volume;
  // /// Latency due to buffering in sink input, see pa_timing_info for details.
  // pa_usec_t buffer_usec;
  // /// Latency of the sink device, see pa_timing_info for details.
  // pa_usec_t sink_usec;
  // /// The resampling method used by this sink input.
  // const char *resample_method;
  // /// Driver name
  // const char *driver;
  // /// Stream muted
  // int mute;
  // /// Property list
  // pa_proplist *proplist;
  // /// Stream corked
  // int corked;
  // /// Stream has volume. If not set, then the meaning of this struct's volume member is unspecified.
  // int has_volume;
  // /// The volume can be set. If not set, the volume can still change even though clients can't control the volume.
  // int volume_writable;
  // /// Stream format information.
  // pa_format_info *format;
}

struct SinkInfo {
  pa: *const sys::pa_sink_info,
}

info! { SinkInfo =>
  /// Name of the sink
  name(&str);
  /// Index of the sink
  index(u32);
  /// Volume of the sink
  volume(Volume);
  /// Some kind of "base" volume that refers to unamplified/unattenuated volume in the context of the output device.
  base_volume(u32);

  // pa_sample_spec sample_spec;        /// Sample spec of this sink
  // pa_channel_map channel_map;        /// Channel map
  // uint32_t owner_module;             /// Index of the owning module of this sink, or PA_INVALID_INDEX.
  // int mute;                          /// Mute switch of the sink
  // uint32_t monitor_source;           /// Index of the monitor source connected to this sink.
  // const char *monitor_source_name;   /// The name of the monitor source.
  // pa_usec_t latency;                 /// Length of queued audio in the output buffer.
  // const char *driver;                /// Driver name
  // pa_sink_flags_t flags;             /// Flags
  // pa_proplist *proplist;             /// Property list
  // pa_usec_t configured_latency;      /// The latency this device has been configured to.
  // pa_sink_state_t state;             /// State
  // uint32_t n_volume_steps;           /// Number of volume steps for sinks which do not support arbitrary volumes.
  // uint32_t card;                     /// Card index, or PA_INVALID_INDEX.
  // uint32_t n_ports;                  /// Number of entries in port array
  // pa_sink_port_info** ports;         /// Array of available ports, or NULL. Array is terminated by an entry set to NULL. The number of entries is stored in n_ports.
  // pa_sink_port_info* active_port;    /// Pointer to active port in the array, or NULL.
  // uint8_t n_formats;                 /// Number of formats supported by the sink.
  // pa_format_info **formats;          /// Array of formats supported by the sink.
}

struct SourceInfo {
  pa: *const sys::pa_source_info,
}

info! { SourceInfo =>
  /// Name of the source
  name(&str);
  /// Index of the source
  index(u32);
  /// Volume of the source
  volume(Volume);
  /// Some kind of "base" volume that refers to unamplified/unattenuated volume in the context of the output device.
  base_volume(u32);

}

struct Volume {
  pa: sys::pa_cvolume,
}

impl Volume {
  pub fn channels(&self) -> u8 { self.pa.channels }
  pub fn values(&self) -> &[u32] { &self.pa.values[..self.channels() as usize] }

  pub fn value_percents(&self) -> Vec<u32> {
    // According to `pactl`, this is how we find the percent:
    // ```
    // ((v * 100 + PA_VOLUME_NORM / 2) / PA_VOLUME_NORM));
    // ```

    self
      .values()
      .iter()
      .map(|v| (v * 100 + sys::PA_VOLUME_NORM / 2) / sys::PA_VOLUME_NORM)
      .collect()
  }
}

impl fmt::Debug for Volume {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Volume")
      .field("channels", &self.channels())
      .field("values", &self.values())
      .field("value_percents", &self.value_percents())
      .finish()
  }
}

impl From<Pulse> for Box<dyn Module> {
  fn from(value: Pulse) -> Self { Box::new(PulseModule::new(value)) }
}

static CONTEXT: Mutex<Option<Arc<Context>>> = Mutex::new(None);

fn context() -> Arc<Context> {
  let mut c = CONTEXT.lock();
  if let Some(ref c) = *c {
    return c.clone();
  }

  let mut l = MainLoop::new();
  let props = PropList::new();
  let mut ctx = Context::new(&mut l, &props);

  let (ready_tx, ready_rx) = crossbeam_channel::bounded(0);

  ctx.set_callback({
    let ctx = ctx.clone();
    move || {
      if ctx.get_state() == ContextState::Ready {
        println!("ready");
        ready_tx.send(()).unwrap();
      }
    }
  });

  ctx.connect();

  std::thread::spawn(move || {
    l.run();
  });

  ready_rx.recv().unwrap();

  let ctx = Arc::new(ctx);

  *c = Some(ctx.clone());
  ctx
}

impl PulseModule {
  pub fn new(spec: Pulse) -> Self { PulseModule { spec } }
}

impl Module for PulseModule {
  fn updater(&self) -> Updater { Updater::None }
  fn layout(&mut self, _layout: &mut cb_bar::Layout) {}
  fn render(&self, _ctx: &mut Render) {}
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let ctx = context();

    let (ready_tx, ready_rx) = crossbeam_channel::bounded(0);
    ctx.get_sink_info_list(move |info| {
      dbg!(&info);
      ready_tx.send(()).unwrap();
    });

    ready_rx.recv().unwrap();
    ctx.set_on_change({
      let c = ctx.clone();
      move || {
        c.get_sink_info_list(move |info| {
          dbg!(&info);
        });
      }
    });

    std::thread::park();
  }
}
