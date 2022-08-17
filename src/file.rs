//! Files stored on camera

use crate::{
  camera::Camera,
  error::Error,
  helper::{chars_to_cow, uninit},
  try_gp_internal, Result,
};
use std::{borrow::Cow, fs, os::unix::io::AsRawFd, path::Path};

/// Represents a path of a file on a camera
pub struct CameraFilePath {
  pub(crate) inner: libgphoto2_sys::CameraFilePath,
}

/// File on a camera
///
/// ## Downloading examples
/// ### In memory
/// ```
/// use gphoto2::Context;
///
/// let camera = Context::new()?.autodetect_camera()?;
/// let file = camera.capture_image()?;
/// let file_data = file.get_in_memory(&camera)?.get_data()?;
/// ```
pub struct CameraFile {
  pub(crate) inner: *mut libgphoto2_sys::CameraFile,
  #[allow(dead_code)]
  // The file must live as long as the camera file to keep the raw file descriptor alive
  file: Option<fs::File>,
}

impl Drop for CameraFile {
  fn drop(&mut self) {
    unsafe {
      libgphoto2_sys::gp_file_unref(self.inner);
    }
  }
}

impl From<libgphoto2_sys::CameraFilePath> for CameraFilePath {
  fn from(file_path: libgphoto2_sys::CameraFilePath) -> Self {
    Self { inner: file_path }
  }
}

impl CameraFilePath {
  /// Get the name of the file's folder
  pub fn folder(&self) -> Cow<str> {
    chars_to_cow(self.inner.folder.as_ptr())
  }

  /// Get the basename of the file (without the folder)
  pub fn name(&self) -> Cow<str> {
    chars_to_cow(self.inner.name.as_ptr())
  }

  fn to_camera_file(&self, camera: &Camera, path: Option<&Path>) -> Result<CameraFile> {
    let camera_file = match path {
      Some(dest_path) => CameraFile::new_file(dest_path)?,
      None => CameraFile::new()?,
    };

    try_gp_internal!(libgphoto2_sys::gp_camera_file_get(
      camera.camera,
      self.inner.folder.as_ptr(),
      self.inner.name.as_ptr(),
      libgphoto2_sys::CameraFileType::GP_FILE_TYPE_NORMAL,
      camera_file.inner,
      camera.context
    ))?;

    Ok(camera_file)
  }

  /// Creates a [`File`] which is downloaded to memory
  pub fn get_in_memory(&self, camera: &Camera) -> Result<CameraFile> {
    self.to_camera_file(camera, None)
  }

  /// Creates a [`File`] which is downloaded to a path on disk
  pub fn download(&self, camera: &Camera, path: &Path) -> Result<CameraFile> {
    self.to_camera_file(camera, Some(path))
  }
}

impl CameraFile {
  pub(crate) fn new() -> Result<Self> {
    let mut camera_file_ptr = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_file_new(&mut camera_file_ptr))?;

    Ok(Self { inner: camera_file_ptr, file: None })
  }

  pub(crate) fn new_file(path: &Path) -> Result<Self> {
    if path.is_file() {
      return Err(Error::new(libgphoto2_sys::GP_ERROR_FILE_EXISTS));
    }

    let file = fs::File::create(path)?;

    let mut camera_file_ptr = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_file_new_from_fd(&mut camera_file_ptr, file.as_raw_fd()))
      .map(|_| Self { inner: camera_file_ptr, file: Some(file) })
  }

  /// Get the data of the file
  pub fn get_data(&self) -> Result<Box<[u8]>> {
    let mut size = unsafe { uninit() };
    let mut data = unsafe { uninit() }; // data from gphoto is returned as i8, but we use it as u8. This might cause errors in future

    try_gp_internal!(libgphoto2_sys::gp_file_get_data_and_size(self.inner, &mut data, &mut size))?;

    let data_slice: Box<[u8]> =
      unsafe { std::slice::from_raw_parts(data as *const u8, size as usize) }.into();

    Ok(data_slice)
  }
}
