//! Cameras and camera events

use crate::{
  abilities::Abilities,
  file::{CameraFile, CameraFilePath},
  filesys::{CameraFS, StorageInfo},
  helper::{as_ref, char_slice_to_cow, chars_to_string, libtool_lock, to_c_string, UninitBox},
  port::PortInfo,
  try_gp_internal,
  widget::{GroupWidget, Widget, WidgetBase},
  Error, Result,
};
use std::{ffi, os::raw::c_char, time::Duration};

/// Event from camera
#[derive(Debug, PartialEq)]
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
/// println!("Camera abilities: {:?}", camera.abilities());
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
/// use gphoto2::{Context, Result, widget::RadioWidget};
///
/// # fn main() -> Result<()> {
/// let context = Context::new()?;
/// let camera = context.autodetect_camera()?;
///
/// let mut iso = camera.config_key::<RadioWidget>("iso")?;
/// iso.set_choice("400")?;
/// camera.set_config(&iso)?;
/// # Ok(())
/// # }
pub struct Camera {
  pub(crate) camera: *mut libgphoto2_sys::Camera,
  pub(crate) context: *mut libgphoto2_sys::GPContext,
}

impl Drop for Camera {
  fn drop(&mut self) {
    // gp_camera_unref -> gp_camera_free -> gp_camera_exit -> libtool
    let _lock = libtool_lock();

    try_gp_internal!(gp_camera_unref(self.camera).unwrap());
    unsafe {
      libgphoto2_sys::gp_context_unref(self.context);
    }
  }
}

as_ref!(Camera -> libgphoto2_sys::Camera, *self.camera);

impl Camera {
  pub(crate) fn new(
    camera: *mut libgphoto2_sys::Camera,
    context: *mut libgphoto2_sys::GPContext,
  ) -> Self {
    unsafe { libgphoto2_sys::gp_context_ref(context) }

    Self { camera, context }
  }

  /// Capture image
  pub fn capture_image(&self) -> Result<CameraFilePath> {
    let mut inner = UninitBox::uninit();

    try_gp_internal!(gp_camera_capture(
      self.camera,
      libgphoto2_sys::CameraCaptureType::GP_CAPTURE_IMAGE,
      inner.as_mut_ptr(),
      self.context
    )?);

    Ok(CameraFilePath { inner: unsafe { inner.assume_init() } })
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
  /// println!("Preview name: {}", image_preview.name());
  /// # Ok(())
  /// # }
  /// ```
  pub fn capture_preview(&self) -> Result<CameraFile> {
    let camera_file = CameraFile::new()?;

    try_gp_internal!(gp_camera_capture_preview(self.camera, camera_file.inner, self.context)?);

    Ok(camera_file)
  }

  /// Get the camera's [`Abilities`]
  ///
  /// The abilities contain information about the driver used, permissions and camera model
  pub fn abilities(&self) -> Abilities {
    let mut inner = UninitBox::uninit();

    try_gp_internal!(gp_camera_get_abilities(self.camera, inner.as_mut_ptr()).unwrap());

    Abilities { inner: unsafe { inner.assume_init() } }
  }

  /// Summary of the cameras model, settings, capabilities, etc.
  pub fn summary(&self) -> Result<String> {
    try_gp_internal!(gp_camera_get_summary(self.camera, &out summary, self.context)?);

    Ok(char_slice_to_cow(&summary.text).into_owned())
  }

  /// Get about information about the camera#
  pub fn about(&self) -> Result<String> {
    try_gp_internal!(gp_camera_get_about(self.camera, &out about, self.context)?);

    Ok(char_slice_to_cow(&about.text).into_owned())
  }

  /// Get the manual of the camera
  ///
  /// Not all cameras support this, and will return NotSupported
  pub fn manual(&self) -> Result<String> {
    try_gp_internal!(gp_camera_get_manual(self.camera, &out manual, self.context)?);

    Ok(char_slice_to_cow(&manual.text).into_owned())
  }

  /// List of storages available on the camera
  pub fn storages(&self) -> Result<Vec<StorageInfo>> {
    try_gp_internal!(gp_camera_get_storageinfo(
      self.camera,
      &out storages_ptr,
      &out storages_len,
      self.context
    )?);

    let storages = unsafe {
      std::slice::from_raw_parts(
        // We can cast pointer safely because StorageInfo is repr(transparent).
        storages_ptr.cast::<StorageInfo>(),
        storages_len.try_into()?,
      )
    };

    let result = storages.to_vec();

    unsafe {
      // Must be freed using libc deallocator rather than Rust one.
      libc::free(storages_ptr.cast());
    }

    Ok(result)
  }

  /// Filesystem actions
  pub fn fs(&self) -> CameraFS<'_> {
    CameraFS::new(self)
  }

  /// Waits for an event on the camera until timeout
  pub fn wait_event(&self, timeout: Duration) -> Result<CameraEvent> {
    use libgphoto2_sys::CameraEventType;

    let duration_milliseconds = timeout.as_millis();

    try_gp_internal!(gp_camera_wait_for_event(
      self.camera,
      duration_milliseconds.try_into()?,
      &out event_type,
      &out event_data,
      self.context
    )?);

    Ok(match event_type {
      CameraEventType::GP_EVENT_UNKNOWN => {
        let s = chars_to_string(event_data.cast::<c_char>());
        unsafe {
          libc::free(event_data);
        }
        CameraEvent::Unknown(s)
      }
      CameraEventType::GP_EVENT_TIMEOUT => CameraEvent::Timeout,
      CameraEventType::GP_EVENT_FILE_ADDED
      | CameraEventType::GP_EVENT_FOLDER_ADDED
      | CameraEventType::GP_EVENT_FILE_CHANGED => {
        let file_path = CameraFilePath {
          inner: Box::new(unsafe { *event_data.cast::<libgphoto2_sys::CameraFilePath>() }),
        };
        unsafe {
          libc::free(event_data);
        }
        match event_type {
          CameraEventType::GP_EVENT_FILE_ADDED => CameraEvent::NewFile(file_path),
          CameraEventType::GP_EVENT_FOLDER_ADDED => CameraEvent::NewFolder(file_path),
          CameraEventType::GP_EVENT_FILE_CHANGED => CameraEvent::FileChanged(file_path),
          _ => unreachable!(),
        }
      }
      CameraEventType::GP_EVENT_CAPTURE_COMPLETE => CameraEvent::CaptureComplete,
    })
  }

  /// Port used to connect to the camera
  pub fn port_info(&self) -> Result<PortInfo<'_>> {
    try_gp_internal!(gp_camera_get_port_info(self.camera, &out port_info)?);

    Ok(unsafe { PortInfo::new(port_info) })
  }

  /// Get the camera configuration
  pub fn config(&self) -> Result<GroupWidget> {
    try_gp_internal!(gp_camera_get_config(self.camera, &out root_widget, self.context)?);

    Widget::new_owned(root_widget).try_into::<GroupWidget>()
  }

  /// Get a single configuration by name.
  /// Pass either a specific widget type as a generic parameter or [`Widget`]
  /// if you're not sure what this config represents.
  pub fn config_key<T: TryFrom<Widget>>(&self, key: &str) -> Result<T>
  where
    Error: From<T::Error>,
  {
    try_gp_internal!(gp_camera_get_single_config(
      self.camera,
      to_c_string!(key),
      &out widget,
      self.context
    )?);

    Ok(Widget::new_owned(widget).try_into()?)
  }

  /// Apply a full config object to the camera.
  pub fn set_all_config(&self, config: &GroupWidget) -> Result<()> {
    try_gp_internal!(gp_camera_set_config(self.camera, config.inner, self.context)?);

    Ok(())
  }

  /// Set a single configuration widget to the camera
  pub fn set_config(&self, config: &WidgetBase) -> Result<()> {
    try_gp_internal!(gp_camera_set_single_config(
      self.camera,
      to_c_string!(config.name()),
      config.inner,
      self.context
    )?);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  fn sample_camera() -> super::Camera {
    crate::sample_context().autodetect_camera().unwrap()
  }

  #[test]
  fn test_abilities() {
    let abilities = sample_camera().abilities();
    insta::assert_debug_snapshot!(abilities);
  }

  #[test]
  fn test_summary() {
    let mut summary = sample_camera().summary().unwrap_or_default();

    // Summary contains dynamic timestamp, find and remove it for snapshotting.
    let prefix = "Date & Time(0x5011):(readwrite) (type=0xffff)";

    summary = summary
      .lines()
      .map(|line| {
        let mut line = line.to_owned();
        if line.starts_with(prefix) {
          line.replace_range(prefix.len().., " (omitted timestamp)");
        }
        line + "\n"
      })
      .collect();

    insta::assert_snapshot!(summary);
  }

  #[test]
  fn test_about() {
    let about = sample_camera().about().unwrap_or_default();
    insta::assert_snapshot!(about);
  }

  #[test]
  fn test_manual() {
    let manual = sample_camera().manual().unwrap_or_default();
    insta::assert_snapshot!(manual);
  }

  #[test]
  fn test_storages() {
    let storages = sample_camera().storages().unwrap();
    insta::assert_debug_snapshot!(storages);
  }

  #[test]
  fn test_fs() {
    use crate::filesys::{CameraFS, FileInfo};
    use std::collections::BTreeMap;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct FolderDbg {
      folders: BTreeMap<String, FolderDbg>,
      files: BTreeMap<String, FileInfo>,
    }

    impl FolderDbg {
      fn collect(fs: &CameraFS, path: &str) -> FolderDbg {
        FolderDbg {
          folders: fs
            .list_folders(path)
            .unwrap()
            .map(|folder_name| {
              let folder = Self::collect(fs, &format!("{path}/{folder_name}"));
              (folder_name, folder)
            })
            .collect(),
          files: fs
            .list_files(path)
            .unwrap()
            .map(|file_name| {
              let mut file_info = fs.file_info(path, &file_name).unwrap();
              // Change timestamp to a constant for the snapshot.
              file_info.inner.file.mtime = 42;
              (file_name, file_info)
            })
            .collect(),
        }
      }
    }

    let camera = sample_camera();

    // capture_image should be checked in the same test as fs, because it
    // modifies the filesystem and it's easier to check both in the same test
    // than dealing with potential race conditions changing the file tree.

    let captured_file_path = camera.capture_image().unwrap();
    insta::assert_debug_snapshot!(captured_file_path);

    let captured_file =
      camera.fs().download(&captured_file_path.folder(), &captured_file_path.name()).unwrap();
    unsafe {
      // Fixup mtime to a constant for the snapshot.
      libgphoto2_sys::gp_file_set_mtime(captured_file.inner, 42);
    }
    insta::assert_debug_snapshot!(captured_file);

    assert_eq!(
      captured_file.get_data().unwrap().as_ref(),
      libgphoto2_sys::test_utils::SAMPLE_IMAGE
    );

    let fs = camera.fs();
    let storages = camera.storages().unwrap();

    let storage_folders = storages
      .iter()
      .map(|storage| {
        let base_dir = storage.base_directory().unwrap();
        let folder_tree = FolderDbg::collect(&fs, &base_dir);
        (base_dir, folder_tree)
      })
      .collect::<BTreeMap<_, _>>();

    insta::assert_debug_snapshot!(storage_folders);
  }

  #[test]
  fn test_camera_events() {
    let camera = sample_camera();

    let event = camera.wait_event(std::time::Duration::from_secs(2)).unwrap();

    insta::assert_debug_snapshot!(event);
  }

  #[test]
  fn test_port_info() {
    let camera = sample_camera();
    let port_info = camera.port_info().unwrap();
    insta::assert_debug_snapshot!(port_info);
  }

  #[test]
  fn test_config() {
    use crate::widget::{DateWidget, TextWidget};

    let widget_tree = sample_camera().config().unwrap();

    // Some widgets represent dynamic information.
    // Find and fix it up before snapshotting.

    widget_tree
      .get_child_by_label("Date & Time")
      .unwrap()
      .try_into::<TextWidget>()
      .unwrap()
      .set_value("(omitted timestamp)")
      .unwrap();

    widget_tree
      .get_child_by_name("datetime")
      .unwrap()
      .try_into::<DateWidget>()
      .unwrap()
      .set_timestamp(42);

    insta::assert_debug_snapshot!(widget_tree);
  }
}
