//! Library context
use crate::{
  abilities::AbilitiesList,
  camera::Camera,
  helper::{as_ref, chars_to_string, libtool_lock, to_c_string},
  list::CameraList,
  list::{CameraDescriptor, CameraListIter},
  port::PortInfoList,
  try_gp_internal, Error, Result,
};
use std::{ffi, rc::Rc};

/// Progress handler trait
pub trait ProgressHandler: 'static {
  /// This method is called when a progress starts.
  ///
  /// It must return a unique ID which is passed to the following functions
  fn start(&mut self, target: f32, message: String) -> u32;

  /// Progress has updated
  fn update(&mut self, id: u32, progress: f32);

  /// Progress has stopped
  fn stop(&mut self, id: u32);
}

/// Context used internally by libgphoto2
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
  progress_handler: Option<Rc<dyn ProgressHandler>>,
}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe { libgphoto2_sys::gp_context_unref(self.inner) }
  }
}

impl Clone for Context {
  fn clone(&self) -> Self {
    unsafe {
      libgphoto2_sys::gp_context_ref(self.inner);
    }

    Self { inner: self.inner, progress_handler: self.progress_handler.clone() }
  }
}

as_ref!(Context -> libgphoto2_sys::GPContext, *self.inner);

impl Context {
  /// Create a new context
  pub fn new() -> Result<Self> {
    #[cfg(feature = "extended_logs")]
    crate::helper::hook_gp_log();

    let context_ptr = unsafe { libgphoto2_sys::gp_context_new() };

    if context_ptr.is_null() {
      return Err(Error::new(libgphoto2_sys::GP_ERROR_NO_MEMORY, None));
    }

    #[cfg(not(feature = "extended_logs"))]
    crate::helper::hook_gp_context_log_func(context_ptr);

    Ok(Self { inner: context_ptr, progress_handler: None })
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

    Ok(Camera::new(camera_ptr, self.clone()))
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

    Ok(Camera::new(camera, self.clone()))
  }

  /// Set context progress functions
  ///
  /// `libgphoto2` allows you to set progress functions to a context, these
  /// allow you to show some progress bars whenever eg. an image is being downloaded.
  ///
  /// # Example
  ///
  /// An example can be found in the examples directory
  pub fn set_progress_functions<H: ProgressHandler>(&mut self, handler: H) {
    use std::os::raw::{c_char, c_float, c_uint, c_void};

    unsafe fn as_handler<H>(data: *mut c_void) -> &'static mut H {
      &mut *data.cast()
    }

    unsafe extern "C" fn start_func<H: ProgressHandler>(
      _ctx: *mut libgphoto2_sys::GPContext,
      target: c_float,
      message: *const c_char,
      data: *mut c_void,
    ) -> c_uint {
      as_handler::<H>(data).start(target, chars_to_string(message))
    }

    unsafe extern "C" fn update_func<H: ProgressHandler>(
      _ctx: *mut libgphoto2_sys::GPContext,
      id: c_uint,
      current: c_float,
      data: *mut c_void,
    ) {
      as_handler::<H>(data).update(id, current)
    }

    unsafe extern "C" fn stop_func<H: ProgressHandler>(
      _ctx: *mut libgphoto2_sys::GPContext,
      id: c_uint,
      data: *mut c_void,
    ) {
      as_handler::<H>(data).stop(id)
    }

    let progress_handler = Rc::new(handler);

    // Now that handler is on the heap, the pointer should be stable.
    // Also, we know that there are and won't be other mutable references to it,
    // so we can safely cast it to a raw *mutable* pointer despite Rc only
    // providing immutable access.
    #[allow(clippy::as_conversions)]
    let data_ptr = Rc::as_ptr(&progress_handler) as *mut c_void;

    unsafe {
      libgphoto2_sys::gp_context_set_progress_funcs(
        self.inner,
        Some(start_func::<H>),
        Some(update_func::<H>),
        Some(stop_func::<H>),
        data_ptr,
      );
    }

    self.progress_handler = Some(progress_handler);
  }
}

#[cfg(all(test, feature = "test"))]
mod tests {
  #[test]
  fn test_list_cameras() {
    let cameras = crate::sample_context().list_cameras().unwrap().collect::<Vec<_>>();
    insta::assert_debug_snapshot!(cameras);
  }

  #[test]
  fn test_progress() {
    use std::fmt::Write;

    let mut context = crate::sample_context();

    #[derive(Default)]
    struct TestProgress {
      log_lines: String,
      next_progress_id: u32,
    }

    impl Drop for TestProgress {
      fn drop(&mut self) {
        insta::assert_snapshot!("progress", self.log_lines);
      }
    }

    impl crate::context::ProgressHandler for TestProgress {
      fn start(&mut self, target: f32, message: String) -> u32 {
        let id = self.next_progress_id;

        self.next_progress_id += 1;
        writeln!(
          self.log_lines,
          "start #{id}: target: {target}, message: {message}",
          message = message.replace(
            libgphoto2_sys::test_utils::libgphoto2_dir().to_str().unwrap(),
            "$LIBGPHOTO2_DIR"
          ),
        )
        .unwrap();
        id
      }

      fn update(&mut self, id: u32, progress: f32) {
        writeln!(self.log_lines, "update #{id}: progress: {progress}").unwrap();
      }

      fn stop(&mut self, id: u32) {
        writeln!(self.log_lines, "stop #{id}").unwrap();
      }
    }

    context.set_progress_functions(TestProgress::default());

    let _ignore = context.list_cameras();
  }
}
