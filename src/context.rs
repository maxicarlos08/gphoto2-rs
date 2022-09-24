//! Library context
use crate::helper::{as_ref, chars_to_string, libtool_lock, to_c_string};
use crate::list::{CameraDescriptor, CameraListIter};
use crate::{
  abilities::AbilitiesList, camera::Camera, list::CameraList, port::PortInfoList, try_gp_internal,
  Error, Result,
};
use std::ffi;

type ProgressStartFunc = Box<dyn Fn(ffi::c_float, String) -> ffi::c_uint>;
type ProgressUpdateFunc = Box<dyn Fn(ffi::c_uint, ffi::c_float)>;
type ProgressStopFunc = Box<dyn Fn(ffi::c_uint)>;

/// Context used internally by gphoto
///
/// ## Example
///
/// ```no_run
/// use gphoto2::{Context, Result};
///
/// # fn main() -> Result<()> {
/// let context = Context::new()?;
///
/// // Use first camera in the camera list
///
/// let camera_desc = context.list_cameras()?.next().ok_or("No cameras found")?;
/// let camera = context.get_camera(&camera_desc)?;
///
/// # Ok(())
/// # }
///
/// ```
pub struct Context {
  pub(crate) inner: *mut libgphoto2_sys::GPContext,
  pub(crate) progress_functions: Option<ContextProgress>,
}

pub(crate) struct ContextProgress {
  start: ProgressStartFunc,
  update: ProgressUpdateFunc,
  stop: ProgressStopFunc,
}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe { libgphoto2_sys::gp_context_unref(self.inner) }
  }
}

as_ref!(Context -> libgphoto2_sys::GPContext, *self.inner);

impl Context {
  /// Create a new context
  pub fn new() -> Result<Self> {
    let context_ptr = unsafe { libgphoto2_sys::gp_context_new() };

    if context_ptr.is_null() {
      return Err(Error::new(libgphoto2_sys::GP_ERROR_NO_MEMORY, None));
    }

    unsafe extern "C" fn log_func(
      _context: *mut libgphoto2_sys::GPContext,
      text: *const libc::c_char,
      log_level_as_ptr: *mut libc::c_void,
    ) {
      let log_level: log::Level = std::mem::transmute(log_level_as_ptr);
      log::log!(target: "gphoto2", log_level, "{}", chars_to_string(text));
    }

    unsafe {
      if log::log_enabled!(log::Level::Error) {
        let log_level_as_ptr = std::mem::transmute(log::Level::Error);

        libgphoto2_sys::gp_context_set_error_func(context_ptr, Some(log_func), log_level_as_ptr);

        // `gp_context_message` seems to be used also for error messages.
        libgphoto2_sys::gp_context_set_message_func(context_ptr, Some(log_func), log_level_as_ptr);
      }

      if log::log_enabled!(log::Level::Info) {
        libgphoto2_sys::gp_context_set_status_func(
          context_ptr,
          Some(log_func),
          std::mem::transmute(log::Level::Info),
        );
      }
    }

    Ok(Self { inner: context_ptr, progress_functions: None })
  }

  /// Lists all available cameras and their ports
  ///
  /// Returns a list of (camera_name, port_path)
  /// which can be used in [`Context::get_camera`].
  pub fn list_cameras(&self) -> Result<CameraListIter> {
    // gp_camera_autodetect -> (gp_port_info_list_load, gp_abilities_list_load, ...) -> libtool
    let _lock = libtool_lock();

    let camera_list = CameraList::new()?;
    try_gp_internal!(gp_camera_autodetect(camera_list.inner, self.inner)?);

    Ok(CameraListIter::new(camera_list))
  }

  /// Auto chooses a camera
  ///
  /// ```no_run
  /// use gphoto2::{Context, Result};
  ///
  /// # fn main() -> Result<()> {
  /// let context = Context::new()?;
  /// if let Ok(camera) = context.autodetect_camera() {
  ///   println!("Successfully autodetected camera '{}'", camera.abilities().model());
  /// } else {
  ///   println!("Could not autodetect camera");
  /// }
  /// # Ok(())
  /// # }
  /// ```
  pub fn autodetect_camera(&self) -> Result<Camera> {
    let _lock = libtool_lock(); // gp_camera_init -> libtool

    try_gp_internal!(gp_camera_new(&out camera_ptr)?);
    try_gp_internal!(gp_camera_init(camera_ptr, self.inner)?);

    Ok(Camera::new(camera_ptr, self.inner))
  }

  /// Initialize a camera knowing its model name and port path
  ///
  /// ```no_run
  /// use gphoto2::{Context, Result};
  ///
  /// # fn main() -> Result<()> {
  /// let context = Context::new()?;
  ///
  /// let camera_desc = context.list_cameras()?.next().ok_or("No cameras found")?;
  /// let camera = context.get_camera(&camera_desc)?;
  ///
  /// # Ok(())
  /// # }
  pub fn get_camera(&self, camera_desc: &CameraDescriptor) -> Result<Camera> {
    let abilities_list = AbilitiesList::new(self)?;
    let port_info_list = PortInfoList::new()?;

    try_gp_internal!(gp_camera_new(&out camera)?);

    try_gp_internal!(let model_index = gp_abilities_list_lookup_model(
      abilities_list.inner,
      to_c_string!(camera_desc.model.as_str())
    )?);

    try_gp_internal!(gp_abilities_list_get_abilities(
      abilities_list.inner,
      model_index,
      &out model_abilities
    )?);
    try_gp_internal!(gp_camera_set_abilities(camera, model_abilities)?);

    try_gp_internal!(let p = gp_port_info_list_lookup_path(
      port_info_list.inner,
      to_c_string!(camera_desc.port.as_str())
    )?);
    let port_info = port_info_list.get_port_info(p)?;
    try_gp_internal!(gp_camera_set_port_info(camera, port_info.inner)?);

    Ok(Camera::new(camera, self.inner))
  }

  /// Set progress functions
  /// TODO: docs
  pub fn set_progress_functions(
    &mut self,
    start: ProgressStartFunc,
    update: ProgressUpdateFunc,
    stop: ProgressStopFunc,
  ) {
    unsafe extern "C" fn start_func(
      _ctx: *mut libgphoto2_sys::GPContext,
      target: ffi::c_float,
      message: *const ffi::c_char,
      data: *mut ffi::c_void,
    ) -> ffi::c_uint {
      call_progress_func!(data, start(target, chars_to_string(message)))
    }

    unsafe extern "C" fn update_func(
      _ctx: *mut libgphoto2_sys::GPContext,
      id: ffi::c_uint,
      current: ffi::c_float,
      data: *mut ffi::c_void,
    ) {
      call_progress_func!(data, update(id, current))
    }

    unsafe extern "C" fn stop_func(
      _ctx: *mut libgphoto2_sys::GPContext,
      id: ffi::c_uint,
      data: *mut ffi::c_void,
    ) {
      call_progress_func!(data, stop(id))
    }

    self.progress_functions = Some(ContextProgress { start, update, stop });

    unsafe {
      #[allow(clippy::as_conversions)]
      libgphoto2_sys::gp_context_set_progress_funcs(
        self.inner,
        Some(start_func),
        Some(update_func),
        Some(stop_func),
        self.progress_functions.as_mut().unwrap() as *mut _ as *mut ffi::c_void,
      )
    }
  }
}

macro_rules! call_progress_func {
    ($data:expr, $function:ident ($($args:expr$(,)?)+)) => {
          ($data.cast::<ContextProgress>().as_ref().unwrap().$function)($($args,)*)
    };
}

pub(self) use call_progress_func;
