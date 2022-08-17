//! Files stored on camera

use crate::{
  camera::Camera,
  error::Error,
  helper::{chars_to_cow, uninit},
  try_gp_internal, Result,
};
use std::{
  borrow::Cow,
  fs,
  os::{raw::{c_char}, unix::io::AsRawFd},
  path::Path,
};

/// Represents a path of a file on a camera
pub struct CameraFilePath {
  pub(crate) inner: libgphoto2_sys::CameraFilePath,
}

/// Type of a file
pub enum FileType {
  /// Preview of an image
  Preview,
  /// Normal fil
  Normal,
  /// Raw data before postprocessing of driver, RAW image files are usually [`FileType::Normal`]
  Raw,
  /// Audio contained in file
  Audio,
  /// Embedded EXIF data of an image
  Exif,
  /// Metadata of a file
  Metadata,
}

/// File on a camera
///
/// ## Downloading examples
/// ### In memory
/// ```no_run
/// use gphoto2::{Context, Result};
///
/// # fn main() -> Result<()> {
/// let context = Context::new()?;
/// let camera = context.autodetect_camera()?;
/// let file = camera.capture_image()?;
/// let file_data = file.get_in_memory(&camera)?.get_data()?;
///
/// # Ok(())
/// # }
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

impl From<libgphoto2_sys::CameraFileType> for FileType {
  fn from(file_type: libgphoto2_sys::CameraFileType) -> Self {
    use libgphoto2_sys::CameraFileType as GPFileType;

    match file_type {
      GPFileType::GP_FILE_TYPE_PREVIEW => Self::Preview,
      GPFileType::GP_FILE_TYPE_NORMAL => Self::Normal,
      GPFileType::GP_FILE_TYPE_RAW => Self::Raw,
      GPFileType::GP_FILE_TYPE_AUDIO => Self::Audio,
      GPFileType::GP_FILE_TYPE_EXIF => Self::Exif,
      GPFileType::GP_FILE_TYPE_METADATA => Self::Metadata,
    }
  }
}

impl Into<libgphoto2_sys::CameraFileType> for FileType {
  fn into(self) -> libgphoto2_sys::CameraFileType {
    use libgphoto2_sys::CameraFileType as GPFileType;

    match self {
      Self::Preview => GPFileType::GP_FILE_TYPE_PREVIEW,
      Self::Normal => GPFileType::GP_FILE_TYPE_NORMAL,
      Self::Raw => GPFileType::GP_FILE_TYPE_RAW,
      Self::Audio => GPFileType::GP_FILE_TYPE_AUDIO,
      Self::Exif => GPFileType::GP_FILE_TYPE_EXIF,
      Self::Metadata => GPFileType::GP_FILE_TYPE_METADATA,
    }
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

  /// Creates a [`CameraFile`] which is downloaded to memory
  pub fn get_in_memory(&self, camera: &Camera) -> Result<CameraFile> {
    self.to_camera_file(camera, None)
  }

  /// Creates a [`CameraFile`] which is downloaded to a path on disk
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

  /// Creates a new camera file from disk
  pub fn new_from_disk(path: &Path) -> Result<Self> {
    let mut camera_file_ptr = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_file_new_from_fd(&mut camera_file_ptr, -1))?;
    try_gp_internal!(libgphoto2_sys::gp_file_open(
      camera_file_ptr,
      path.to_str().ok_or("File path invalid")?.as_ptr() as *const c_char
    ))?;

    Ok(Self { inner: camera_file_ptr, file: None })
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

  /// File name
  pub fn name(&self) -> Result<Cow<str>> {
    let mut file_name = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_file_get_name(self.inner, &mut file_name))?;

    Ok(chars_to_cow(file_name))
  }

  /// File mime type
  pub fn mime(&self) -> Result<Cow<str>> {
    let mut mime = unsafe { uninit() };

    try_gp_internal!(libgphoto2_sys::gp_file_get_mime_type(self.inner, &mut mime))?;

    Ok(chars_to_cow(mime))
  }
}
