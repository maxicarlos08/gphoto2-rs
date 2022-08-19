//! Cameras and camera events

use crate::{
  abilities::Abilities,
  file::{CameraFile, CameraFilePath},
  filesys::{CameraFS, StorageInfo},
  helper::{camera_text_to_str, chars_to_cow, uninit},
  port::PortInfo,
  try_gp_internal,
  widget::{Widget, WidgetType},
  Result,
};
use std::{borrow::Cow, ffi, marker::PhantomData, os::raw::c_char, time::Duration};

/// Event from camera
#[derive(Debug)]
pub enum CameraEvent {
  /// Unknown event
  Unknown(String),
  /// Timeout, no event,
  Timeout,
  /// New file was added
  NewFile(CameraFilePath),
  ///  File has changed
  FileChanged(CameraFilePath),
  /// New folder was added
  ///
  /// In the filepath, [`folder`](CameraFilePath::folder) is the parent folder
  /// and [`name`](CameraFilePath::name) is the name of the created folder
  NewFolder(CameraFilePath),
  /// Capture completed
  CaptureComplete,
}

/// Represents a camera
///
/// Cameras can only be created from a [`Context`](crate::Context) by using either
/// [`Context::autodetect_camera`](crate::Context::autodetect_camera) to let gphoto
/// automatically choose a camera or [`Context::get_camera`](crate::Context::get_camera)
/// to get a specific camera.
///
/// ## Capturing images
///
/// This example captures an image without downloading it to disk
///
/// ```no_run
/// use gphoto2::{Context, Result};
///
/// # fn main() -> Result<()> {
/// let context = Context::new()?;
/// let camera = context.autodetect_camera()?;
///
/// // Get some basic information about the camera
/// println!("Camera abilities: {:?}", camera.abilities()?);
/// println!("Camera summary: {}", camera.summary()?);
///
/// // Capture an image
/// let image = camera.capture_image()?;
///
/// // Image can be downloaded using image.download(&camera, download_path)
/// # Ok(())
/// # }
/// ```
///
/// ## Configuring the camera
///
/// Each camera has its own configuration, this is an example configuration
/// for my Nikon D3400 (set the iso to 400).
///
/// ```no_run
/// use gphoto2::{Context, Result, widget::WidgetValue};
///
/// # fn main() -> Result<()> {
/// let context = Context::new()?;
/// let camera = context.autodetect_camera()?;
///
/// let mut iso = camera.config_key("iso")?;
/// iso.set_value(WidgetValue::Menu("400".into()))?;
/// camera.set_config(&iso)?;
/// # Ok(())
/// # }
pub struct Camera<'a> {
  pub(crate) camera: *mut libgphoto2_sys::Camera,
  pub(crate) context: *mut libgphoto2_sys::GPContext,
  _phantom: PhantomData<&'a ffi::c_void>,
}

impl Drop for Camera<'_> {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_camera_unref(self.camera);
    }
  }
}

impl<'a> Camera<'a> {
  pub(crate) fn new(
    camera: *mut libgphoto2_sys::Camera,
    context: *mut libgphoto2_sys::GPContext,
  ) -> Self {
    Self { camera, context, _phantom: PhantomData }
  }

  /// Capture image
  pub fn capture_image(&self) -> Result<CameraFilePath> {
    let mut file_path_ptr = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_capture(
      self.camera,
      libgphoto2_sys::CameraCaptureType::GP_CAPTURE_IMAGE,
      &mut file_path_ptr,
      self.context
    ))?;

    Ok(file_path_ptr.into())
  }

  /// Capture a preview image
  ///
  /// ```no_run
  /// use gphoto2::{Context, Result};
  ///
  /// # fn main() -> Result<()> {
  /// let context = Context::new()?;
  /// let camera = context.autodetect_camera()?;
  ///
  /// let image_preview = camera.capture_preview()?;
  /// println!("Preview name: {}", image_preview.name()?);
  /// # Ok(())
  /// # }
  /// ```
  pub fn capture_preview(&self) -> Result<CameraFile> {
    let camera_file = CameraFile::new()?;

    try_gp_internal!(libgphoto2_sys::gp_camera_capture_preview(
      self.camera,
      camera_file.inner,
      self.context
    ))?;

    Ok(camera_file)
  }

  /// Get the camera's [`Abilities`]
  ///
  /// The abilities contain information about the driver used, permissions and camera model
  pub fn abilities(&self) -> Result<Abilities> {
    let mut abilities = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_abilities(self.camera, &mut abilities))?;

    Ok(abilities.into())
  }

  /// Summary of the cameras model, settings, capabilities, etc.
  pub fn summary(&self) -> Result<Cow<str>> {
    let mut summary = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_summary(
      self.camera,
      &mut summary,
      self.context
    ))?;

    Ok(camera_text_to_str(summary))
  }

  /// Get about information about the camera#
  pub fn about(&self) -> Result<Cow<str>> {
    let mut about = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_about(self.camera, &mut about, self.context))?;

    Ok(camera_text_to_str(about))
  }

  /// Get the manual of the camera
  ///
  /// Not all cameras support this, and will return NotSupported
  pub fn manual(&self) -> Result<Cow<str>> {
    let mut manual = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_manual(self.camera, &mut manual, self.context))?;

    Ok(camera_text_to_str(manual))
  }

  /// List of storages available on the camera
  pub fn storages(&self) -> Result<Vec<StorageInfo>> {
    let mut storages_ptr = unsafe { uninit() };
    let mut storages_len = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_storageinfo(
      self.camera,
      &mut storages_ptr,
      &mut storages_len,
      self.context
    ))?;

    Ok(
      unsafe { Vec::from_raw_parts(storages_ptr, storages_len as usize, storages_len as usize) }
        .into_iter()
        .map(StorageInfo::new)
        .collect(),
    )
  }

  /// Filesystem actions
  pub fn fs(&'a self) -> CameraFS<'a> {
    CameraFS::new(self)
  }

  /// Waits for an event on the camera until timeout
  pub fn wait_event(&self, timeout: Duration) -> Result<CameraEvent> {
    use libgphoto2_sys::CameraEventType;

    let duration_milliseconds = timeout.as_millis();

    let mut event_type = unsafe { uninit() };
    let mut event_data = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_wait_for_event(
      self.camera,
      duration_milliseconds as i32,
      &mut event_type,
      &mut event_data,
      self.context
    ))?;

    Ok(match event_type {
      CameraEventType::GP_EVENT_UNKNOWN => {
        let data = chars_to_cow(event_data as *const c_char);
        CameraEvent::Unknown(data.to_string())
      }
      CameraEventType::GP_EVENT_TIMEOUT => CameraEvent::Timeout,
      CameraEventType::GP_EVENT_FILE_ADDED => {
        let file = event_data as *const libgphoto2_sys::CameraFilePath;
        CameraEvent::NewFile(CameraFilePath { inner: unsafe { *file } })
      }
      CameraEventType::GP_EVENT_FOLDER_ADDED => {
        let folder = event_data as *const libgphoto2_sys::CameraFilePath;
        CameraEvent::NewFolder(CameraFilePath { inner: unsafe { *folder } })
      }
      CameraEventType::GP_EVENT_FILE_CHANGED => {
        let changed_file = event_data as *const libgphoto2_sys::CameraFilePath;
        CameraEvent::FileChanged(CameraFilePath { inner: unsafe { *changed_file } })
      }
      CameraEventType::GP_EVENT_CAPTURE_COMPLETE => CameraEvent::CaptureComplete,
    })
  }

  /// Port used to connect to the camera
  pub fn port_info(&self) -> Result<PortInfo> {
    let mut port_info = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_port_info(self.camera, &mut port_info))?;

    Ok(PortInfo { inner: port_info })
  }

  /// Get the camera configuration
  pub fn config(&self) -> Result<Widget<'a>> {
    let mut root_widget = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_camera_get_config(
      self.camera,
      &mut root_widget,
      self.context
    ))?;

    Ok(Widget::new(root_widget))
  }

  /// Get a single configuration by name
  pub fn config_key(&self, key: &str) -> Result<Widget<'a>> {
    let mut widget = unsafe { uninit() };

    let key = ffi::CString::new(key)?;

    try_gp_internal!(libgphoto2_sys::gp_camera_get_single_config(
      self.camera,
      key.as_ptr() as *const c_char,
      &mut widget,
      self.context
    ))?;

    Ok(Widget::new(widget))
  }

  /// Apply a full config object to the camera.
  ///
  /// The configuration widget must be of type [`Window`](crate::widget::WidgetType::Window)
  pub fn set_all_config(&self, config: &Widget) -> Result<()> {
    if !matches!(config.widget_type()?, WidgetType::Window) {
      Err("Full config object must be of type Window")?;
    }

    try_gp_internal!(libgphoto2_sys::gp_camera_set_config(
      self.camera,
      config.inner,
      self.context
    ))?;

    Ok(())
  }

  /// Set a single configuration widget to the camera
  pub fn set_config(&self, config: &Widget) -> Result<()> {
    let name = ffi::CString::new(&config.name()?[..])?;

    try_gp_internal!(libgphoto2_sys::gp_camera_set_single_config(
      self.camera,
      name.as_ptr() as *const c_char,
      config.inner,
      self.context
    ))?;

    Ok(())
  }
}
