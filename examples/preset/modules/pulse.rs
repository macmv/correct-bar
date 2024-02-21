use correct_bar::bar::{Color, ModuleImpl, Updater};
use crossbeam_channel::Receiver;
use libpulse_sys as sys;
use parking_lot::Mutex;
use std::{
  ffi::{c_void, CStr, CString},
  fmt, ptr,
};

#[derive(Clone)]
pub struct Pulse {
  color: Color,
  recv:  Receiver<()>,
}

struct MainLoop {
  pa: *mut sys::pa_mainloop,
}

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
  ($name:ident($($arg_name:ident: $arg_ty:ty)*), $sys:ident, dyn FnOnce($ty:ty), |$info:ident: $info_ty:ty| $constructor:expr $(, $eol:tt)?) => {
    pub fn $name(&self, $($arg_name: $arg_ty,)* custom: impl FnOnce($ty) + Send + 'static) {
      extern "C" fn callback(
        _ctx: *mut sys::pa_context,
        $info: $info_ty,
        $($eol: i32,)?
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

  // Don't call this twice lol.
  pub fn set_callback(&mut self, custom: impl Fn() + Send + 'static) {
    static CALLBACK: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);
    extern "C" fn cb(_ctx: *mut sys::pa_context, _userdata: *mut c_void) {
      let cb = CALLBACK.lock();
      if let Some(cb) = &*cb {
        cb();
      }
    }

    CALLBACK.lock().replace(Box::new(custom));
    unsafe {
      sys::pa_context_set_state_callback(self.pa, Some(cb), ptr::null_mut());
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
  callback!(
    get_source_info(),
    pa_context_get_source_output_info_list,
    dyn FnOnce(&SourceOutputInfo),
    |info: *const sys::pa_source_output_info| &SourceOutputInfo { pa: info },
    _eol
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

impl Pulse {
  pub fn new(color: Color) -> Self {
    let (tx, rx) = crossbeam_channel::bounded(16);

    std::thread::spawn(move || {
      let mut l = MainLoop::new();

      let props = PropList::new();

      let mut ctx = Context::new(&mut l, &props);

      ctx.set_callback({
        let ctx = ctx.clone();
        move || {
          if ctx.get_state() == ContextState::Ready {
            println!("ready");

            ctx.get_server_info({
              let ctx = ctx.clone();
              move |info| {
                println!("got server info: {:?}", info);

                ctx.get_source_info(|info| {
                  println!("got source info: {:?}", info.pa);
                });
              }
            });
          }
        }
      });

      ctx.connect();

      tx.send(()).unwrap();

      l.run();
    });

    Pulse { color, recv: rx }
  }
}

impl ModuleImpl for Pulse {
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    ctx.draw_text("fooo", self.color);
    ctx.draw_text("%", self.color);
  }
  fn updater(&self) -> Updater { Updater::Channel(self.recv.clone()) }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}
