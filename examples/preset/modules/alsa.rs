// SAFETY: This entire module uses the alsa_sys api directly! Edit with caution!

use alsa_sys as alsa;
use correct_bar::bar::{Color, ModuleImpl, Updater};
use std::{
  ffi::{c_int, CStr, CString},
  fmt,
  time::Duration,
};

#[derive(Clone)]
pub struct ALSA {
  card:  c_int,
  color: Color,
}

macro_rules! check {
  ( $expr:expr) => {{
    let res = $expr;
    if res < 0 {
      Err(ALSAError { code: res })
    } else {
      Ok(res)
    }
  }};
}

struct ALSAError {
  code: i32,
}

type Result<T> = std::result::Result<T, ALSAError>;

impl fmt::Debug for ALSAError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = unsafe { alsa::snd_strerror(self.code) };
    if s.is_null() {
      write!(f, "ALSA error: unknown error code {}", self.code)
    } else {
      write!(f, "ALSA error: {}", unsafe { CStr::from_ptr(s) }.to_str().unwrap())
    }
  }
}

struct ControlCardInfo(*mut alsa::snd_ctl_card_info_t);

impl fmt::Debug for ControlCardInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "control card info here") }
}

impl Drop for ControlCardInfo {
  fn drop(&mut self) {
    unsafe {
      alsa::snd_ctl_card_info_free(self.0);
    }
  }
}

struct Control(*mut alsa::snd_ctl_t);

struct ControlElemList(*mut alsa::snd_ctl_elem_list_t);
struct ControlElem<'a> {
  list:  &'a ControlElemList,
  // SAFETY: This index must be valid
  index: u32,
}

impl Control {
  pub fn new(id: i32) -> Result<Self> {
    unsafe {
      let name = CString::new(format!("hw:{id}")).unwrap();

      let mut ctl: *mut alsa::snd_ctl_t = std::ptr::null_mut();
      check!(alsa::snd_ctl_open(&mut ctl, name.as_ptr(), 1))?;

      Ok(Control(ctl))
    }
  }

  pub fn info(&self) -> Result<ControlCardInfo> {
    unsafe {
      let mut info: *mut alsa::snd_ctl_card_info_t = std::ptr::null_mut();
      check!(alsa::snd_ctl_card_info_malloc(&mut info))?;
      check!(alsa::snd_ctl_card_info(self.0, info))?;

      Ok(ControlCardInfo(info))
    }
  }

  pub fn elems(&self) -> Result<ControlElemList> {
    unsafe {
      let mut list: *mut alsa::snd_ctl_elem_list_t = std::ptr::null_mut();
      check!(alsa::snd_ctl_elem_list_malloc(&mut list))?;

      // sets the length of the list
      check!(alsa::snd_ctl_elem_list(self.0, list))?;
      let len = alsa::snd_ctl_elem_list_get_count(list);

      // allocate space for identifiers
      check!(alsa::snd_ctl_elem_list_alloc_space(list, len))?;
      // call this again, to copy in the identifiers, which will let us actually use
      // the list.
      check!(alsa::snd_ctl_elem_list(self.0, list))?;

      Ok(ControlElemList(list))
    }
  }
}

impl Drop for ControlElemList {
  fn drop(&mut self) {
    unsafe {
      // free space for identifiers and the list itself.
      alsa::snd_ctl_elem_list_free_space(self.0);
      alsa::snd_ctl_elem_list_free(self.0);
    }
  }
}

impl ControlElemList {
  pub fn len(&self) -> u32 { unsafe { alsa::snd_ctl_elem_list_get_count(self.0) } }
  pub fn iter(&self) -> ControlElemIter {
    ControlElemIter { index: 0, len: self.len(), list: &self }
  }
  pub fn to_vec(&self) -> Vec<ControlElem> { self.iter().collect() }
}

impl ControlElem<'_> {
  pub fn name(&self) -> String {
    unsafe {
      let ptr = alsa::snd_ctl_elem_list_get_name(self.list.0, self.index);
      if ptr.is_null() {
        panic!("got null ptr from elem list name");
      }
      CStr::from_ptr(ptr).to_str().unwrap().to_string()
    }
  }
  pub fn device(&self) -> u32 {
    unsafe { alsa::snd_ctl_elem_list_get_device(self.list.0, self.index) }
  }
  /*
  pub fn interface(&self) -> Interface {
    unsafe {
      check!(alsa::snd_ctl_elem_list_get_device(self.list.0, self.index)).unwrap();
    }
  }
  */
}

impl fmt::Debug for ControlElem<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ControlElem")
      .field("name", &self.name())
      .field("device", &self.device())
      .finish()
  }
}

struct ControlElemIter<'a> {
  index: u32,
  len:   u32,
  list:  &'a ControlElemList,
}

impl<'a> Iterator for ControlElemIter<'a> {
  type Item = ControlElem<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.index >= self.len {
      return None;
    }

    let ret = ControlElem { list: self.list, index: self.index };
    self.index += 1;
    Some(ret)
  }
}

impl ALSA {
  pub fn new() -> Self { Self::new_inner().unwrap() }

  fn new_inner() -> Result<Self> {
    unsafe {
      let name = CString::new("Generic").unwrap();
      let id = check!(alsa::snd_card_get_index(name.as_ptr()))?;

      let card = id;

      let control = Control::new(id)?;

      dbg!(control.elems()?.to_vec());

      /*
      use alsa::{
        pcm::{Access, Format, HwParams, State, PCM},
        Direction, ValueOr,
      };

      // Open default playback device
      let pcm = PCM::new("default", Direction::Playback, false).unwrap();

      // Set hardware parameters: 48000 Hz / Mono / 16 bit
      let hwp = HwParams::any(&pcm).unwrap();
      hwp.set_channels(1).unwrap();
      hwp.set_rate(48000, ValueOr::Nearest).unwrap();
      hwp.set_format(Format::s16()).unwrap();
      hwp.set_access(Access::RWInterleaved).unwrap();
      pcm.hw_params(&hwp).unwrap();
      let io = pcm.io_i16().unwrap();

      // Make sure we don't start the stream too early
      let hwp = pcm.hw_params_current().unwrap();
      let swp = pcm.sw_params_current().unwrap();
      swp.set_start_threshold(hwp.get_buffer_size().unwrap()).unwrap();
      pcm.sw_params(&swp).unwrap();

      // Make a sine wave
      let mut buf = [0i16; 1024];
      for (i, a) in buf.iter_mut().enumerate() {
        *a = ((i as f32 * 2.0 * ::std::f32::consts::PI / 128.0).sin() * 8192.0) as i16
      }

      // Play it back for 2 seconds.
      for _ in 0..2 * 48000 / 1024 {
        assert_eq!(io.writei(&buf[..]).unwrap(), 1024);
      }

      // In case the buffer was larger than 2 seconds, start the stream manually.
      if pcm.state() != State::Running {
        pcm.start().unwrap()
      };
      // Wait for the stream to finish playback.
      pcm.drain().unwrap();
      */

      Ok(ALSA { card, color: Color { r: 100, g: 255, b: 128 } })
    }
  }
}

impl ModuleImpl for ALSA {
  fn render(&self, ctx: &mut correct_bar::bar::RenderContext) {
    // foo
    ctx.draw_text("100%", self.color);
  }
  fn updater(&self) -> Updater { Updater::Every(Duration::from_secs(1)) }
  fn box_clone(&self) -> Box<dyn ModuleImpl + Send + Sync> { Box::new(self.clone()) }
}
